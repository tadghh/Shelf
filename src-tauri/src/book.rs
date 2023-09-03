use std::{ fs::{ OpenOptions, File }, path::{ Path, PathBuf }, io::BufReader };
use serde::{ Deserialize, Serialize };
use epub::doc::EpubDoc;
use regex::Regex;
use xmltree::Element;

use crate::book::util::{ chunk_binary_search_index_load, base64_encode_file, check_epub_resource };
use crate::xml::extract_image_source;

use self::bookio::write_cover_image;
use self::util::sanitize_windows_filename;

pub mod bookio;
pub mod util;

/// This is used for organization
struct BookCache {
    books: Vec<Book>,
    json_path: String,
}

impl BookCache {
    /// Used to update the location of the book_cache.json file
    fn update_path(&mut self, new_json_path: String) {
        self.json_path = new_json_path;
    }
    /// Used to update the contents of the book_cache.json file
    fn update_books(&mut self, new_books: Vec<Book>) {
        self.books = new_books;
    }
}

static mut BOOK_JSON: BookCache = BookCache {
    books: Vec::new(),
    json_path: String::new(),
};

/// Used for handling books on the front end
#[derive(Serialize, Deserialize, Debug)]
pub struct Book {
    cover_location: String,
    book_location: String,
    title: String,
}

/// Loads a books data and returns a base64 encoding of it
/// This is done to get around CORS issues, here is where we take advantage of binary search
///
/// # Arguments
///
/// * `title` - The title of the book to load
///
#[tauri::command]
pub fn load_book(title: String) -> Result<String, String> {
    unsafe {
        let book_cache: &String = &BOOK_JSON.json_path;

        if Path::new(&book_cache).exists() {
            let file = OpenOptions::new().read(true).write(true).create(true).open(book_cache);

            BOOK_JSON.update_books(match serde_json::from_reader(BufReader::new(file.unwrap())) {
                Ok(data) => data,
                Err(_) => Vec::new(),
            });

            let books = &BOOK_JSON.books;
            let book_index = chunk_binary_search_index_load(books, &title);
            if let Some(book) = books.get(book_index.unwrap()) {
                // Accessing the book at the specified index
                return Ok(book.book_location.to_string());
            } else {
                println!("Invalid index");
            }
        } else {
            return Err("JSON File missing".to_string());
        }
    }

    Err("Error occured".to_string())
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
    if
        let Some(cover_id) = check_epub_resource(
            Regex::new(r"(?i)cover").unwrap(),
            Regex::new(r"application/xhtml\+xml").unwrap(),
            &epub_resources,
            &mut doc
        )
    {
        let resource = doc.get_resource(&cover_id);

        let file_content = &resource.unwrap().0;
        let buffer_str = String::from_utf8_lossy(file_content);
        let root = Element::parse(buffer_str.as_bytes()).expect("Failed to parse XML");

        if let Some(image_element_src) = extract_image_source(&root) {
            if
                let Some(src) = check_epub_resource(
                    Regex::new(&image_element_src).unwrap(),
                    Regex::new(r"image/jpeg").unwrap(),
                    &epub_resources,
                    &mut doc
                )
            {
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
fn create_cover(book_directory: String, write_directory: &PathBuf) -> Result<PathBuf, String> {
    let mut doc = EpubDoc::new(book_directory).map_err(|err|
        format!("Error opening EpubDoc: {}", err)
    )?;

    let epub_resources = doc.resources.clone();

    //Base filename off the books title

    let cover_path = &write_directory.join(
        sanitize_windows_filename(format!("{}.jpg", doc.mdata("title").unwrap()))
    );

    //The below get_cover method only looks for a certain structure of cover image
    if doc.get_cover().is_some() {
        if let Err(err) = write_cover_image(doc.get_cover(), cover_path) {
            return Ok(err.to_path_buf());
        }
    } else {
        //Look for the cover_id in the epub, we are just looking for any property containing the word cover
        //This is because EpubDoc looks for an exact string, and some epubs dont contain it
        // let mimetype = r"image/jpeg";
        if
            let Some(cover_id) = check_epub_resource(
                Regex::new(r"(?i)cover").unwrap(),
                Regex::new(r"image/jpeg").unwrap(),
                &epub_resources,
                &mut doc
            )
        {
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
