use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
    sync::Mutex,
};

use crate::{
    book::{
        bookio::{get_book_cover_image, write_cover_image, BookError},
        util::{
            check_epub_resource, chunk_binary_search_index_load, get_cache_dir,
            sanitize_windows_filename,
        },
    },
    book_worker::BookWorker,
};
use epub::doc::EpubDoc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use tauri::State;
use xmltree::Element;

use crate::xml::extract_image_source;

/// This is used for organization
pub struct BookCache {
    books: Option<Vec<Book>>,
    json_path: String,
}

impl BookCache {
    /// Used to update the location of the book_cache.json file
    // fn update_path(&mut self, new_json_path: String) {
    //     self.json_path = new_json_path;
    // }
    /// Used to update the contents of the book_cache.json file
    pub fn new(books: Option<Vec<Book>>) -> BookCache {
        BookCache {
            books,
            json_path: get_cache_dir()
                .join(env!("CACHE_F_NAME"))
                .to_string_lossy()
                .to_string(),
        }
    }
    pub fn update_books(&mut self, new_books: Vec<Book>) {
        self.books = Some(new_books);
    }

    pub fn get_json_path(&self) -> &String {
        &self.json_path
    }
    pub fn get_books(&self) -> &Vec<Book> {
        self.books.as_ref().unwrap()
    }
    pub fn update_json_path(&mut self, json_path: String) {
        self.json_path = json_path;
    }
}
// pub static mut BOOK_JSON: BookCache = BookCache {
//     books: Vec::new(),
//     json_path: String::new(),
// };

/// Used for handling books on the front end
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Book {
    cover_location: String,
    book_location: String,
    title: String,
}
impl Book {
    pub fn new(cover_location: Option<String>, book_location: String, title: String) -> Book {
        // Clean book title
        let final_cover_location = match cover_location {
            Some(cover_loc) => cover_loc,
            None => {
                let epub_doc = EpubDoc::new(&book_location)
                    .map_err(|err| format!("Error opening EpubDoc: {}", err))
                    .unwrap();

                let mut covers_directory = get_cache_dir();
                covers_directory.push(env!("COVER_IMAGE_FOLDER_NAME"));

                // why why why, am I supposed to make another object before the ensures everything is g? (unwrap)
                let mut cover_name = epub_doc.mdata("title").unwrap();
                cover_name.push_str(".jpg");

                let cover_path = &covers_directory.join(sanitize_windows_filename(cover_name));
                let final_shit = match get_book_cover_image(epub_doc) {
                    Ok(cover_data) => match write_cover_image(cover_data, cover_path) {
                        // I need the path as a string plz
                        Ok(cover_path) => match cover_path.to_str() {
                            Some(cover_string) => cover_string.to_string(),
                            // even though we could write the cover image the 'path' was somehow None
                            None => env!("DEFAULT_COVER_NAME").to_string(),
                        },
                        Err(_) => {
                            println!(
                                "Wrote the file successfully but it was written into the abyss"
                            );

                            env!("DEFAULT_COVER_NAME").to_string()
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                        env!("DEFAULT_COVER_NAME").to_string()
                    }
                };
                final_shit
            }
        };

        Book {
            cover_location: final_cover_location,
            book_location,
            title,
        }
    }
    pub fn get_title(&self) -> &String {
        &self.title
    }
    pub fn get_cover_location(&self) -> &String {
        &self.cover_location
    }
    pub fn get_book_location(&self) -> &String {
        &self.book_location
    }
}

/// Looks for the books url inside the json file, returning its path
///
/// # Arguments
///
/// * `title` - The title of the book to load
///
#[tauri::command]
pub fn load_book(title: String, state: State<'_, Mutex<BookWorker>>) -> Option<Book> {
    let mut book_worker = state.lock().unwrap();
    let book_json_cache = book_worker.get_book_cache().get_json_path().clone();

    if Path::new(&book_json_cache).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(book_json_cache);

        let new_books: Vec<Book> = match from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        book_worker.update_book_cache(new_books);
        let books = book_worker.get_book_cache().get_books();
        let book_index = chunk_binary_search_index_load(books, &title);
        //TODO This should be a result (this method, we cant load "None" book)
        match book_index {
            Some(book_index) => return books.get(book_index).cloned(),
            None => return None,
        }
    } else {
        println!("JSON File missing");
    }

    None
}

/// The current crate used for handling Epubs has some issues with finding covers for uniquely structured books
///
/// # Arguments
///
/// * `doc` - The epub documents itself
/// * `cover_path` - The path to write the cover data too
///
pub fn unique_find_cover(
    mut doc: EpubDoc<BufReader<File>>,
) -> Result<(Vec<u8>, String), BookError> {
    let epub_resources = doc.resources.clone();

    //The scenario where the cover_id has a xhtml file set as its property
    match check_epub_resource(
        Regex::new(r"(?i)cover").unwrap(),
        Regex::new(r"image/jpeg").unwrap(),
        &epub_resources,
        &mut doc,
    ) {
        Some(cover_id) => match doc.get_resource(&cover_id) {
            Some(cover_data) => Ok(cover_data),
            None => Err(BookError::NoUniqueCover),
        },
        None => match check_epub_resource(
            Regex::new(r"(?i)cover").unwrap(),
            Regex::new(r"application/xhtml\+xml").unwrap(),
            &epub_resources,
            &mut doc,
        ) {
            Some(unique_cover_id) => {
                let resource: Option<(Vec<u8>, String)> = doc.get_resource(&unique_cover_id);

                let file_content = &resource.unwrap().0;
                let buffer_str = String::from_utf8_lossy(file_content);
                let root = Element::parse(buffer_str.as_bytes()).expect("Failed to parse XML");
                match extract_image_source(&root) {
                    Some(image_element_src) => match check_epub_resource(
                        Regex::new(&image_element_src).unwrap(),
                        Regex::new(r"image/jpeg").unwrap(),
                        &epub_resources,
                        &mut doc,
                    ) {
                        Some(src) => match doc.get_resource(&src) {
                            Some(cover_data) => Ok(cover_data),
                            None => Err(BookError::BadCoverData),
                        },
                        None => Err(BookError::NoUniqueCover),
                    },
                    None => Err(BookError::NoUniqueCover),
                }
            }
            None => Err(BookError::NoUniqueCover),
        },
    }
}

// pub fn create_cover(book_directory: String, write_directory: &PathBuf) -> Result<PathBuf, ()> {
//     let mut doc =
//         EpubDoc::new(book_directory).map_err(|err| format!("Error opening EpubDoc: {}", err))?;

//     //The below get_cover method only looks for a certain structure of cover image

//     //Look for the cover_id in the epub, we are just looking for any property containing the word cover
//     //This is because EpubDoc looks for an exact string, and some epubs dont contain it
//     // let mimetype = r"image/jpeg";
//     if let Some(cover_id) = check_epub_resource(
//         Regex::new(r"(?i)cover").unwrap(),
//         Regex::new(r"image/jpeg").unwrap(),
//         &epub_resources,
//         &mut doc,
//     ) {
//         let cover: Option<(Vec<u8>, String)> = doc.get_resource(&cover_id);

//         if let Err(err) = write_cover_image(cover, cover_path) {
//             return Ok(err.to_path_buf());
//         }
//     } else if let Err(err) = find_cover(doc, cover_path) {
//         return Ok(err);
//     }

//     Ok(cover_path.to_path_buf())
// }
