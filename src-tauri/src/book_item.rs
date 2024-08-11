use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::{
    book::{
        bookio::write_cover_image,
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
    pub fn new(cover_location: String, book_location: String, title: String) -> Book {
        Book {
            book_location,
            cover_location,
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
    // let book_cache: &String = &BOOK_JSON.json_path;

    if Path::new(&book_json_cache).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(book_json_cache);
        //.update_books(new_books)

        //let book_clone = book_worker;

        let new_books: Vec<Book> = match from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        book_worker.update_book_cache(new_books);
        //*book_worker = book_clone.clone();
        let books = book_worker.get_book_cache().get_books();
        let book_index = chunk_binary_search_index_load(books, &title);

        if let Some(book) = books.get(book_index.unwrap()) {
            // Accessing the book at the specified index
            return Some(book.clone());
        } else {
            println!("Invalid index");
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
fn find_cover(mut doc: EpubDoc<BufReader<File>>, cover_path: &PathBuf) -> Result<(), PathBuf> {
    let epub_resources = doc.resources.clone();

    //The scenario where the cover_id has a xhtml file set as its property
    if let Some(cover_id) = check_epub_resource(
        Regex::new(r"(?i)cover").unwrap(),
        Regex::new(r"application/xhtml\+xml").unwrap(),
        &epub_resources,
        &mut doc,
    ) {
        let resource = doc.get_resource(&cover_id);

        let file_content = &resource.unwrap().0;
        let buffer_str = String::from_utf8_lossy(file_content);
        let root = Element::parse(buffer_str.as_bytes()).expect("Failed to parse XML");

        if let Some(image_element_src) = extract_image_source(&root) {
            if let Some(src) = check_epub_resource(
                Regex::new(&image_element_src).unwrap(),
                Regex::new(r"image/jpeg").unwrap(),
                &epub_resources,
                &mut doc,
            ) {
                write_cover_image(doc.get_resource(&src), cover_path)?;
            }
        }
    }

    Ok(())
}

/// Creates the cover for the given book, returning the path to it in the cache folder, otherwise returning the fallback image
///
/// # Arguments
///
/// * `book_directory` - The directory of the book
/// * `write_directory` - The path to write the cover data too
///
pub fn create_cover(book_directory: String, write_directory: &PathBuf) -> Result<PathBuf, String> {
    let mut doc =
        EpubDoc::new(book_directory).map_err(|err| format!("Error opening EpubDoc: {}", err))?;

    let epub_resources = doc.resources.clone();

    //Base filename off the books title

    let cover_path = &write_directory.join(sanitize_windows_filename(format!(
        "{}.jpg",
        doc.mdata("title").unwrap()
    )));

    //The below get_cover method only looks for a certain structure of cover image
    if doc.get_cover().is_some() {
        //TODO wth is this
        if let Err(err) = write_cover_image(doc.get_cover(), cover_path) {
            return Ok(err.to_path_buf());
        }
    } else {
        //Look for the cover_id in the epub, we are just looking for any property containing the word cover
        //This is because EpubDoc looks for an exact string, and some epubs dont contain it
        // let mimetype = r"image/jpeg";
        if let Some(cover_id) = check_epub_resource(
            Regex::new(r"(?i)cover").unwrap(),
            Regex::new(r"image/jpeg").unwrap(),
            &epub_resources,
            &mut doc,
        ) {
            let cover: Option<(Vec<u8>, String)> = doc.get_resource(&cover_id);

            if let Err(err) = write_cover_image(cover, cover_path) {
                return Ok(err.to_path_buf());
            }
        } else if let Err(err) = find_cover(doc, cover_path) {
            return Ok(err);
        }
    }

    Ok(cover_path.to_path_buf())
}
