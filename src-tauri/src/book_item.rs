use std::{
    fs::{create_dir_all, File},
    io::BufReader,
    path::PathBuf,
    sync::Mutex,
};

use crate::{
    book::{
        bookio::{get_book_cover_image, write_cover_image, BookError},
        util::{check_epub_resource, current_context, get_cover_dir, sanitize_windows_filename},
    },
    book_worker::BookWorker,
    database::get_db,
};
use epub::doc::EpubDoc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteQueryResult, Sqlite};
use tauri::{api::path::app_cache_dir, State};
use tokio::runtime::Runtime;
use xmltree::Element;

use crate::xml::extract_image_source;

// TODO just make it empty vector instead of usig option
/// This is used for organization
pub struct BookCache {
    books: Option<Vec<Book>>,
}

impl BookCache {
    /// Used to update the contents of the book_cache.json file
    pub fn new(books: Option<Vec<Book>>) -> BookCache {
        BookCache { books }
    }

    pub fn update_books(&mut self, new_books: Option<Vec<Book>>) {
        self.books = new_books;
    }
    pub fn get_book_amount(&self) -> usize {
        match &self.books {
            Some(books) => books.len(),
            None => 0,
        }
    }
    pub fn get_books(&self) -> Option<&Vec<Book>> {
        self.books.as_ref()
    }
    pub fn find_by_title(&self, title: &str) -> Option<&Book> {
        self.books.as_ref()?.iter().find(|book| book.title == title)
    }
}

/// Used for handling books on the front end#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Book {
    cover_location: Option<String>,
    book_location: String,
    title: String,
}

// Authors are creative right? surely there arent two books with the same title
impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

impl Book {
    pub fn new(cover_location: Option<String>, book_location: String, title: String) -> Book {
        // Tries to write the cover image to 'cover_cache'
        // Otherwise uses default.jpg from /public
        let final_cover_location = match cover_location {
            Some(cover_loc) => Some(cover_loc),
            None => {
                let epub_doc = EpubDoc::new(&book_location)
                    .map_err(|err| format!("Error opening EpubDoc: {}", err))
                    .unwrap();

                let covers_directory = get_cover_dir();

                // why why why, am I supposed to make another object before the ensures everything is g? (unwrap)
                let mut cover_name = epub_doc.mdata("title").unwrap();
                cover_name.push_str(".jpg");
                let cover_path_san = sanitize_windows_filename(cover_name.clone());
                let cover_path = &covers_directory.join(sanitize_windows_filename(cover_name));

                match get_book_cover_image(epub_doc) {
                    Ok(cover_data) => match write_cover_image(cover_data, cover_path) {
                        // I need the path as a string plz
                        Ok(_) => Some(cover_path_san.clone()),
                        Err(_) => {
                            println!(
                                "Wrote the file successfully but it was written into the abyss, perhaps a folder is missing from the 'cover_path'"
                            );

                            None
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                        None
                    }
                }
            }
        };

        Book {
            cover_location: final_cover_location,
            book_location,
            title,
        }
    }

    fn get_cover_dir(&self) -> PathBuf {
        let mut cache_dir =
            app_cache_dir(&current_context()).expect("Failed to get cache directory");
        cache_dir.push("cache");
        cache_dir.push(env!("COVER_IMAGE_FOLDER_NAME"));
        if let Err(err) = create_dir_all(&cache_dir) {
            eprintln!("Error creating {:?} directory: {:?}", err, cache_dir);
        }

        cache_dir
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_cover_location(&self) -> String {
        match &self.cover_location {
            Some(cover) => self
                .get_cover_dir()
                .join(cover)
                .to_string_lossy()
                .to_string(),
            None => env!("DEFAULT_COVER_NAME").to_string(),
        }
    }

    pub fn get_cover_filename(&self) -> &str {
        println!("get {:?}", self.cover_location);
        match &self.cover_location {
            Some(cover) => cover,
            None => env!("DEFAULT_COVER_NAME"),
        }
    }

    pub fn get_book_location(&self) -> &String {
        &self.book_location
    }
}

#[tauri::command]
pub fn get_cover_location_command(book: Book) -> String {
    book.get_cover_location()
}

/// Looks for the books url inside the json file, returning its path
///
/// # Arguments
///
/// * `title` - The title of the book to load
///
#[tauri::command]
pub fn load_book(title: String, state: State<'_, Mutex<BookWorker>>) -> Option<Book> {
    let book_worker = state.lock().unwrap();
    let book_cache = book_worker.get_book_cache();
    match book_cache.find_by_title(&title) {
        Some(book) => Some(book.clone()),
        None => None,
    }
}

pub fn get_all_books() -> Result<Vec<Book>, sqlx::Error> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        let books = sqlx::query_as::<_, Book>("SELECT * FROM books")
            .fetch_all(get_db())
            .await?;
        Ok(books)
    })
}

pub fn drop_books_from_table() -> Result<SqliteQueryResult, sqlx::Error> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        // dont format this line
        Ok(sqlx::query("DELETE FROM books").execute(get_db()).await?)
    })
}

pub fn create_books_table() -> Result<SqliteQueryResult, sqlx::Error> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        // dont format this line
        Ok(sqlx::query("CREATE TABLE IF NOT EXISTS books (id INTEGER PRIMARY KEY AUTOINCREMENT, cover_location TEXT NOT NULL, book_location TEXT NOT NULL, title TEXT NOT NULL);").execute(get_db()).await?)    })
}

// TODO should add a checksum to the db along with the books
// I imagine indexing the checksum would be faster in comparison to ILIKE
pub fn get_book_on_name(name: String) -> Result<Option<Book>, sqlx::Error> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        match sqlx::query_as("SELECT * FROM books WHERE title ILIKE $1")
            .bind(&name)
            .fetch_optional(get_db())
            .await
        {
            Ok(book) => Ok(book),
            Err(e) => Err(e),
        }
    })
}

pub async fn insert_book_db(new_book: Book) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO books (cover_location, book_location, title) VALUES ($1, $2, $3)")
        .bind(new_book.get_cover_filename())
        .bind(new_book.get_book_location())
        .bind(new_book.get_title())
        .execute(get_db())
        .await?;
    Ok(())
}

pub fn insert_book_db_batch(new_book_batch: &Vec<Book>) -> Result<(), sqlx::Error> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        let mut query_builder: sqlx::QueryBuilder<Sqlite> =
            sqlx::QueryBuilder::new("INSERT INTO books (cover_location, book_location, title) ");

        //TODO Might hit bind limits if users 'accumulates' books
        query_builder.push_values(new_book_batch.iter(), |mut b, book| {
            b.push_bind(book.get_cover_filename())
                .push_bind(book.get_book_location())
                .push_bind(book.get_title());
        });

        let query = query_builder.build();
        query.execute(get_db()).await?;

        Ok(())
    })
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
