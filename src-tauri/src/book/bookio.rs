use epub::doc::EpubDoc;
use std::{
    fs::{self, create_dir_all, File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};
use tauri::State;

use crate::{
    book::util::{chunk_binary_search_index, get_cache_dir},
    book_item::{create_cover, get_json_path, Book},
    book_worker::BookWorker,
    shelf::{get_cache_file_name, get_configuration_option, get_cover_image_folder_name},
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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
            }
            Ok(file) => file,
        };
        if file.write_all(&data.0).is_err() {
            return Err(path);
        }
    }

    Ok(())
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
                let new_book = Book::new(
                    cover_location.to_string_lossy().to_string(),
                    item.replace('\\', "/"),
                    title,
                );
                Some(new_book)
            } else {
                None // Skip this book and continue with the next one
            }
        })
        .collect();

    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| a.get_title().cmp(&b.get_title()));

    sorted_books
}

/// Initializes the books and loading them from the users provided directory, if the book_cache file is missing the all epubs will be read
/// Otherwise only books missing from the Static vector will be initialized
#[tauri::command]
pub fn initialize_books(state: State<'_, Mutex<BookWorker>>) -> Option<Vec<Book>> {
    let start_time = Instant::now();

    let mut file_changes = false;

    let mut book_json: Vec<Book>;

    let json_path: String = get_json_path();

    //Need to add support for book_location being an array of string
    let dir = match get_configuration_option("book_location".to_string()) {
        Some(val) => val,
        None => {
            return None;
        }
    };

    if !Path::new(&dir).exists() {
        return None;
    }

    let epubs: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            match path {
                p if p.is_file() && p.extension().unwrap() == "epub" => {
                    Some(p.to_str().unwrap().to_owned())
                }
                _ => None,
            }
        })
        .collect();

    let epub_amount = epubs.len();

    let mut covers_directory = get_cache_dir();

    covers_directory.push(get_cover_image_folder_name());

    if let Err(err) = create_dir_all(&covers_directory) {
        eprintln!("Error creating cover directory: {:?}", err);
    }
    let mut pecker = state.lock().unwrap();
    let fuck = get_json_path().clone();
    // unsafe {
    if fuck != json_path {
        pecker.update_cache_json_path(json_path.clone());
    }
    // }

    if Path::new(&json_path).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&json_path);

        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };

        let current_length = book_json.len();

        if current_length != epub_amount {
            let new_books: Vec<(Book, usize)> = epubs
                .par_iter()
                .filter_map(|item| {
                    let item_normalized = item.replace('\\', "/");

                    if let Some(title) = EpubDoc::new(&item_normalized)
                        .expect("The epub path was bad")
                        .mdata("title")
                    {
                        if let Some(index) = chunk_binary_search_index(&book_json, &title) {
                            let new_book = Book::new(
                                create_cover(item_normalized.to_string(), &covers_directory)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                                item_normalized,
                                title,
                            );

                            return Some((new_book, index));
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();

            if new_books.len() != 0 {
                let mut index_offset = 0;
                for (book, index) in new_books {
                    book_json.insert(index + index_offset, book);
                    index_offset += 1;
                }

                file_changes = true;
            }
        }
    } else {
        book_json = create_book_vec(&epubs, &covers_directory);
        file_changes = true;
    }

    if file_changes {
        let file = File::create(json_path).expect("JSON path should be defined, and a valid path");

        serde_json::to_writer_pretty(file, &book_json).expect("The book JSON should exist");
    }

    println!("Execution time: {} ms", start_time.elapsed().as_millis());

    Some(book_json)
}

pub fn initialize_books_start() -> Option<Vec<Book>> {
    let start_time = Instant::now();

    let mut file_changes = false;

    let book_json: Vec<Book>;

    let json_path: String = get_json_path();

    //Need to add support for book_location being an array of string
    let dir = match get_configuration_option("book_location".to_string()) {
        Some(val) => val,
        None => {
            return None;
        }
    };

    if !Path::new(&dir).exists() {
        return None;
    }

    let epubs: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            match path {
                p if p.is_file() && p.extension().unwrap() == "epub" => {
                    Some(p.to_str().unwrap().to_owned())
                }
                _ => None,
            }
        })
        .collect();

    let mut covers_directory = get_cache_dir();

    covers_directory.push(get_cover_image_folder_name());

    if let Err(err) = create_dir_all(&covers_directory) {
        eprintln!("Error creating cover directory: {:?}", err);
    }

    if Path::new(&json_path).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&json_path);

        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
    } else {
        book_json = create_book_vec(&epubs, &covers_directory);
        file_changes = true;
    }

    if file_changes {
        let file = File::create(json_path).expect("JSON path should be defined, and a valid path");

        serde_json::to_writer_pretty(file, &book_json).expect("The book JSON should exist");
    }

    println!("Execution time: {} ms", start_time.elapsed().as_millis());

    Some(book_json)
}
