use std::{path::Path, time::Instant, env, fs::{self, OpenOptions, File}, io::BufReader};
use epub::doc::EpubDoc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{shelf::{get_configuration_option, get_cover_image_folder_name, get_cache_file_name}, book::{BOOK_JSON, util::chunk_binary_search_index}};

use super::{Book, create_cover};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};


//

//Checks if a directory exists and if not its path is created
fn create_directory(path: &String, new_folder_name: &str) -> String {
    let created_dir = Path::new(&path).join(new_folder_name);
    if !Path::new(&created_dir).exists() {
        if let Err(err) = std::fs::create_dir_all(&created_dir) {
            eprintln!("Failed to create folder: {}", err);
        }
    }
    return created_dir.to_string_lossy().replace('\\', "/");
}

//This creates the vector to be written to the json file
pub fn create_book_vec(items: &Vec<String>, write_directory: &String) -> Vec<Book> {
    let books: Vec<Book> = items
        .par_iter()
        .filter_map(|item| {
            let title = EpubDoc::new(item).unwrap().mdata("title").unwrap();

            let new_book = Book {
                cover_location: create_cover(item.to_string(), write_directory),
                book_location: item.replace('\\', "/"),
                title,
            };

            Some(new_book)
        })
        .collect();

    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| a.title.cmp(&b.title));

    sorted_books
}

#[tauri::command]
pub fn create_covers() -> Option<Vec<Book>> {

    //file name to long
    let start_time = Instant::now();

    let mut file_changes = false;

    //Get rust directory
    let home_dir = &env::current_dir()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");

    let json_path = format!("{}/{}", &home_dir, get_cache_file_name());
    let dir = match get_configuration_option("book_folder_location".to_string()) {
        Some(val) => val,
        None => "".to_string(),
    };
    if dir == *"null" || dir == *"" {
        return None;
    }
    let paths = fs::read_dir(&dir);

    //Load epubs from the provided directory in the frontend, currently the dashboards component
    let epubs: Vec<String> = paths
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

    let mut book_json: Vec<Book>;

    //We need this folder to load cover images, otherwise images need to base64 encoded and thats to hacky
    let public_directory = create_directory(home_dir, "cache");
    let covers_directory = create_directory(
        &public_directory,
        get_cover_image_folder_name(),
    );

    unsafe {
        if BOOK_JSON.json_path != json_path {
            BOOK_JSON.update_path(json_path.to_string());
        }
    }

    if Path::new(&json_path).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&json_path);
        println!("{:?}", json_path);
        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        let book_json_len = Arc::new(AtomicUsize::new(book_json.len()));

        let book_json_test = Arc::new(Mutex::new(book_json));

        epubs.par_iter().for_each(|item| {
            let item_normalized = item.replace('\\', "/");
            let title = EpubDoc::new(&item_normalized)
                .unwrap()
                .mdata("title")
                .unwrap();

            let mut book_json_guard = book_json_test.lock().unwrap();
            let index = chunk_binary_search_index(&book_json_guard, &title);
            match index {
                Some(index) => {
                    let new_book = Book {
                        cover_location: create_cover(item_normalized.to_string(), &covers_directory),
                        book_location: item_normalized,
                        title,
                    };
                    book_json_guard.insert(index, new_book);
                    book_json_len.fetch_sub(1, Ordering::SeqCst);
                }
                None => println!("There was no index")
            }

        });

        book_json = Arc::try_unwrap(book_json_test)
            .unwrap()
            .into_inner()
            .unwrap();
        let final_length = book_json_len.load(Ordering::SeqCst);

        //if the lengths are dff bool it
        if book_json.len() != final_length {
            file_changes = true;
        }
    } else {
        book_json = create_book_vec(&epubs, &covers_directory);
        file_changes = true;
    }
    if file_changes {
        let file = File::create(json_path).expect("Json path should be defined and a valid path");

        serde_json::to_writer_pretty(file, &book_json).expect("The book json file should exist")
    }

    let elapsed_time = start_time.elapsed();
    println!("Execution time: {} ms", elapsed_time.as_millis());

    Some(book_json)
}