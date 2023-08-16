use epub::doc::EpubDoc;
use tauri::api::path::cache_dir;
use std::{
    fs::{ self, File, OpenOptions },
    io::{ BufReader, Write },
    path::{ Path, PathBuf },
    sync::{ atomic::{ AtomicUsize, Ordering }, Arc, Mutex },
    time::Instant,
};

use rayon::prelude::{ IntoParallelRefIterator, ParallelIterator };
use crate::{
    book::{ util::chunk_binary_search_index, BOOK_JSON, get_home_dir, Book, create_cover },
    shelf::{ get_cache_file_name, get_configuration_option, get_settings_name },
};

/// Writes the cover image to the specified path
///
/// # Arguments
///
/// * `data` - A vector containing the image data
/// * `path` - A string representing the path to write to
///
pub fn write_cover_image(data: Option<(Vec<u8>, String)>, path: &PathBuf) -> Result<(), &PathBuf> {
    if let Some(data) = data {
        let mut file = match File::create(path) {
            Err(..) => {
                return Err(path);
                // return Err(format!("{}/{}", get_home_dir(), "error.jpg"));
            }
            Ok(file) => file,
        };
        if file.write_all(&data.0).is_err() {
            // return Err(format!("{}/{}", get_home_dir(), "error.jpg"));
            return Err(path);
        }
    }

    Ok(())
}

/// Creates a directory and returns its path
///
/// # Arguments
///
/// * `path` - A string representing the directory to be made
/// * `new_folder_name` - A string, the new folders name
///
fn create_directory(path: &String, new_folder_name: &str) -> String {
    let created_dir = Path::new(&path).join(new_folder_name);
    if !Path::new(&created_dir).exists() {
        if let Err(err) = std::fs::create_dir_all(&created_dir) {
            eprintln!("Failed to create folder: {}", err);
        }
    }
    return created_dir.to_string_lossy().replace('\\', "/");
}

/// Creates the default settings file if none exists
pub fn create_default_settings_file() {
    let home_dir = get_home_dir();
    let settings_path = format!("{}/{}", home_dir, get_settings_name());

    // Check if the file already exists
    if fs::metadata(&settings_path).is_err() {
        // File doesn't exist, create a new one with default values
        let default_settings =
            r#"
            book_folder_location=None
            endless_scroll=false
        "#;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&settings_path)
            .expect("Failed to create settings file");

        file.write_all(default_settings.as_bytes()).expect(
            "Failed to write default settings to file"
        );
    }
}

/// Creates a vector containing all the books and returns a a vector of book objects, here we also create the covers
/// The returned json is sorted alphabetically so we can use binary sort when there are a large number of books
///
/// # Arguments
///
/// * `items` - A vector containing the book directories
/// * `write_directory` - A string representing the path to write to
///
pub fn create_book_vec(items: &Vec<String>, write_directory: &PathBuf) -> Vec<Book> {
    let books: Vec<Book> = items
        .par_iter()
        .filter_map(|item| {
            let title = EpubDoc::new(item).unwrap().mdata("title").unwrap();

            if let Ok(cover_location) = create_cover(item.to_string(), write_directory) {
                let new_book = Book {
                    cover_location: cover_location.to_string_lossy().to_string(),
                    book_location: item.replace('\\', "/"),
                    title,
                };
                Some(new_book)
            } else {
                None // Skip this book and continue with the next one
            }
        })
        .collect();

    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| a.title.cmp(&b.title));

    sorted_books
}

/// Initializes the books and loading them from the users provided directory, if the book_cache file is missing the all epubs will be read
/// Otherwise only books missing from the Static vector will be initialized
#[tauri::command]
pub fn initialize_books() -> Option<Vec<Book>> {
    let start_time = Instant::now();

    let mut file_changes = false;

    let mut book_json: Vec<Book>;

    let json_path = format!("{}/{}", get_home_dir(), get_cache_file_name());
    let dir = match get_configuration_option("book_folder_location".to_string()) {
        Some(val) => val,
        None => {
            return None;
        }
    };

    //Load epubs from the provided directory in the frontend, currently the dashboards component
    let epubs: Vec<String> = fs
        ::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "epub" {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();

    let cache_directory = create_directory(&get_home_dir(), "cache");
    let covers_directory = &cache_dir().unwrap();

    unsafe {
        if BOOK_JSON.json_path != json_path {
            BOOK_JSON.update_path(json_path.to_string());
        }
    }

    if Path::new(&json_path).exists() {
        let file = OpenOptions::new().read(true).write(true).create(true).open(&json_path);

        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };

        let current_length = &book_json.len();
        if current_length != &epubs.len() {
            let book_json_len = Arc::new(AtomicUsize::new(book_json.len()));
            let book_json_test = Arc::new(Mutex::new(book_json));

            epubs.par_iter().for_each(|item| {
                let item_normalized = item.replace('\\', "/");
                let title = EpubDoc::new(&item_normalized).unwrap().mdata("title").unwrap();
                let mut book_json_guard = book_json_test.lock().unwrap();
                let index = chunk_binary_search_index(&book_json_guard, &title);
                match index {
                    Some(index) => {
                        let new_book = Book {
                            cover_location: create_cover(
                                item_normalized.to_string(),
                                covers_directory
                            )
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                            book_location: item_normalized,
                            title,
                        };
                        book_json_guard.insert(index, new_book);
                        book_json_len.fetch_sub(1, Ordering::SeqCst);
                    }
                    None => println!("There was no index"),
                }
            });
            book_json = Arc::try_unwrap(book_json_test).unwrap().into_inner().unwrap();
            let final_length = book_json_len.load(Ordering::SeqCst);

            if book_json.len() != final_length {
                file_changes = true;
            }
        }
    } else {
        book_json = create_book_vec(&epubs, covers_directory);
        println!("{} length", book_json.len());
        file_changes = true;
    }
    if file_changes {
        let file = File::create(json_path).expect("JSON path should be defined, and a valid path");

        serde_json::to_writer_pretty(file, &book_json).expect("The book JSON should exist");
    }

    let elapsed_time = start_time.elapsed();
    println!("Execution time: {} ms", elapsed_time.as_millis());

    Some(book_json)
}
