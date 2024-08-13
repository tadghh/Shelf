use core::fmt;
use epub::doc::EpubDoc;
use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
    sync::Mutex,
};
use tauri::State;

use crate::{
    book_item::{unique_find_cover, Book},
    book_worker::BookWorker,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

/// Writes the cover image to the specified path
///
/// # Arguments
///
/// * `data` - A vector containing the image data
/// * `path` - A string representing the path to write to
///
pub fn write_cover_image(data: (Vec<u8>, String), path: &PathBuf) -> Result<&PathBuf, ()> {
    let (bytes, _) = data;

    match File::create(path) {
        Err(..) => return Err(()),
        Ok(mut file) => {
            if file.write_all(&bytes).is_err() {
                return Err(());
            }
        }
    }

    Ok(path)
}

/// Creates a vector containing all the books and returns a a vector of book objects, here we also create the covers
/// The returned json is sorted alphabetically so we can use binary sort when there are a large number of books
///
/// # Arguments
///
/// * `items` - A vector containing the book directories
/// * `write_directory` - A string representing the path to write to
///
pub fn create_book_vec(items: &Vec<String>) -> Vec<Book> {
    let books: Vec<Book> = items
        .par_iter()
        .filter_map(|item| {
            if let Ok(book) = EpubDoc::new(item) {
                let title = book.mdata("title")?;
                Some(Book::new(None, item.to_string(), title))
            } else {
                None
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
    // let start_time = Instant::now();

    // let mut file_changes = false;

    // let mut book_json: Vec<Book>;
    let book_worker = state.lock().unwrap();

    book_worker.initialize_books()

    // let dir = match book_worker.get_application_settings().get("book_location") {
    //     Some(val) => val,
    //     None => {
    //         return None;
    //     }
    // };

    // if !Path::new(&dir).exists() {
    //     return None;
    // }

    // let epubs: Vec<String> = fs::read_dir(dir)
    //     .unwrap()
    //     .filter_map(|entry| {
    //         let path = entry.unwrap().path();
    //         match path {
    //             p if p.is_file() && p.extension()? == "epub" => {
    //                 Some(p.to_str().unwrap().to_owned())
    //             }
    //             _ => None,
    //         }
    //     })
    //     .collect();

    // let epub_amount = epubs.len();

    // // TODO make sure default is used if this is none (not in this exact context)

    // if Path::new(&json_path).exists() {
    //     let file = OpenOptions::new()
    //         .read(true)
    //         .write(true)
    //         .create(true)
    //         .open(&json_path);

    //     book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
    //         Ok(data) => data,
    //         Err(_) => Vec::new(),
    //     };

    //     let current_length = book_json.len();

    //     if current_length != epub_amount {
    //         let new_books: Vec<(Book, usize)> = epubs
    //             .par_iter()
    //             .filter_map(|item| {
    //                 let item_normalized = item.replace('\\', "/");

    //                 match EpubDoc::new(&item_normalized) {
    //                     Ok(ebook) => {
    //                         let book_title = ebook.mdata("title")?;
    //                         let index = chunk_binary_search_index(&book_json, &book_title)?;

    //                         let new_book = Book::new(None, item_normalized, book_title);

    //                         return Some((new_book, index));
    //                     }
    //                     Err(e) => {
    //                         println!("Book creation failed with: {}", e);

    //                         return None;
    //                     }
    //                 }
    //             })
    //             .collect::<Vec<_>>();

    //         if new_books.len() != 0 {
    //             let mut index_offset = 0;
    //             for (book, index) in new_books {
    //                 book_json.insert(index + index_offset, book);
    //                 index_offset += 1;
    //             }

    //             file_changes = true;
    //         }
    //     }
    // } else {
    //     book_json = create_book_vec(&epubs);
    //     file_changes = true;
    // }

    // if file_changes {
    //     let file = File::create(json_path).expect("JSON path should be defined, and a valid path");

    //     serde_json::to_writer_pretty(file, &book_json).expect("The book JSON should exist");
    // }

    // println!("Execution time: {} ms", start_time.elapsed().as_millis());

    // Some(json_path)
}

// pub fn initialize_books_start(
//     cover_folder_name: String,
//     json_path: String,
//     book_location: &String,
// ) -> Option<Vec<Book>> {
//     let start_time = Instant::now();

//     let mut file_changes = false;

//     let book_json: Vec<Book>;

//     if !Path::new(&book_location).exists() {
//         println!("Start failed no book dir");

//         return None;
//     }

//     let epubs: Vec<String> = fs::read_dir(book_location)
//         .unwrap()
//         .filter_map(|entry| {
//             let path = entry.unwrap().path();
//             match path {
//                 p if p.is_file() && p.extension().unwrap() == "epub" => {
//                     Some(p.to_str().unwrap().to_owned())
//                 }
//                 _ => None,
//             }
//         })
//         .collect();

//     let mut covers_directory = get_cache_dir();

//     covers_directory.push(cover_folder_name);

//     if let Err(err) = create_dir_all(&covers_directory) {
//         eprintln!("Error creating cover directory: {:?}", err);
//     }

//     if Path::new(&json_path).exists() {
//         let file = OpenOptions::new()
//             .read(true)
//             .write(true)
//             .create(true)
//             .open(&json_path);

//         book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
//             Ok(data) => data,
//             Err(_) => Vec::new(),
//         };
//     } else {
//         book_json = create_book_vec(&epubs);
//         file_changes = true;
//     }

//     if file_changes {
//         let file = File::create(json_path).expect("JSON path should be defined, and a valid path");

//         serde_json::to_writer_pretty(file, &book_json).expect("The book JSON should exist");
//     }

//     println!("Execution time: {} ms", start_time.elapsed().as_millis());

//     Some(book_json)
// }

#[derive(Debug)]
pub enum BookError {
    NoUniqueCover,
    ResourceNotFound,
    XmlParseError,
    IOError,
    BadCoverData,
}

impl fmt::Display for BookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BookError::NoUniqueCover => write!(f, "Unique cover not found."),
            BookError::ResourceNotFound => write!(f, "Resource not found."),
            BookError::XmlParseError => write!(f, "Failed to parse XML."),
            BookError::IOError => write!(f, "I/O error occurred."),
            BookError::BadCoverData => write!(f, "Cover data missing or corrupted"),
        }
    }
}

impl std::error::Error for BookError {}

pub fn get_book_cover_image(
    mut doc: EpubDoc<BufReader<File>>,
) -> Result<(Vec<u8>, std::string::String), BookError> {
    // let epub_resources = doc.resources.clone();

    //Base filename off the books title

    // let cover_path = &write_directory.join(sanitize_windows_filename(format!(
    //     "{}.jpg",
    //     doc.mdata("title").unwrap()
    // )));

    //The below get_cover method only looks for a certain structure of cover image
    match doc.get_cover() {
        Some(cover_data) => {
            //TODO log failed cover creations, because a expected cover exists
            Ok(cover_data)
        }
        None => {
            // no giving up a unique cover may still exist
            match unique_find_cover(doc) {
                Ok(cover_data) => Ok(cover_data),
                Err(err) => Err(err),
            }
        }
    }
    // if doc.get_cover().is_some() {
    //     //TODO wth is this
    //     if let Err(err) = write_cover_image(doc.get_cover(), cover_path) {
    //         return Ok(err.to_path_buf());
    //     }
    // } else {
    //     //Look for the cover_id in the epub, we are just looking for any property containing the word cover
    //     //This is because EpubDoc looks for an exact string, and some epubs dont contain it
    //     // let mimetype = r"image/jpeg";
    //     match check_epub_resource(
    //         Regex::new(r"(?i)cover").unwrap(),
    //         Regex::new(r"image/jpeg").unwrap(),
    //         &epub_resources,
    //         &mut doc,
    //     ) {
    //         Some(cover_id) => {
    //             let cover: Option<(Vec<u8>, String)> = doc.get_resource(&cover_id);
    //         }
    //         None => match unique_find_cover(doc, cover_path) {
    //             Ok() => todo!(),
    //             Err(_) => todo!(),
    //         },
    //     }
    // if let Some(cover_id) = check_epub_resource(
    //     Regex::new(r"(?i)cover").unwrap(),
    //     Regex::new(r"image/jpeg").unwrap(),
    //     &epub_resources,
    //     &mut doc,
    // ) {
    //     let cover: Option<(Vec<u8>, String)> = doc.get_resource(&cover_id);

    //     if let Err(err) = write_cover_image(cover, cover_path) {
    //         return Ok(err.to_path_buf());
    //     }
    // } else if let Err(err) = find_cover(doc, cover_path) {
    //     return Ok(err);
    // }
    // }

    // Ok(cover_path.to_path_buf())
}
