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
    println!("{:?} items handed to create new", items.len());
    let books: Vec<Book> = items
        .par_iter()
        .filter_map(|item| {
            let item_normalized = item.replace('\\', "/");

            match EpubDoc::new(&item_normalized) {
                Ok(ebook) => {
                    let book_title = ebook.mdata("title")?;

                    let new_book = Book::new(None, item_normalized, book_title);

                    Some(new_book)
                }
                Err(e) => {
                    println!("Book creation failed with: {}", e);

                    None
                }
            }
        })
        .collect();

    let mut sorted_books = books;

    // TODO this might cause issues, only some data is sorted on the front end
    sorted_books.sort_by(|a, b| a.get_title().cmp(&b.get_title()));

    sorted_books
}

/// Initializes the books and loading them from the users provided directory, if the book_cache file is missing the all epubs will be read
/// Otherwise only books missing from the Static vector will be initialized
#[tauri::command]
pub fn initialize_books(state: State<'_, Mutex<BookWorker>>) -> Option<Vec<Book>> {
    let mut book_worker = state.lock().unwrap();

    book_worker.initialize_books()
}

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
    match doc.get_cover() {
        Some(cover_data) => {
            //TODO log failed cover creations, because a expected cover exists
            Ok(cover_data)
        }
        None => {
            // Handles epubs with incorrectly named covers images
            match unique_find_cover(doc) {
                Ok(cover_data) => Ok(cover_data),
                Err(err) => Err(err),
            }
        }
    }
}
