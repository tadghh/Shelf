use epub::doc::EpubDoc;
use regex::Regex;

use sqlx::Sqlite;
use tauri::{api::path::app_cache_dir, generate_context, Config};

use crate::book_item::Book;

use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, create_dir_all, File},
    io::BufReader,
    path::{Path, PathBuf},
};

/// Finds a chunk in the dataset that starts with the same letter as the key, returning the found value
/// One this chunk is found we binary search within that section, theoretically faster
///
/// # Arguments
///
/// * `dataset` - A dataset sorted in alphabetical order
/// * `key` - The key to look for
///
pub fn chunk_binary_search_index(dataset: &Vec<Book>, key: &String) -> Option<usize> {
    let title = key.to_string();
    //handel lower case
    let low = dataset
        .iter()
        .position(|b| b.get_title()[..1] == title[..1]);

    if let Some(index) = low {
        let mut high = dataset
            .iter()
            .rposition(|b| b.get_title()[..1] == title[..1])
            .unwrap();
        let mut unwrapped_low = index;
        while unwrapped_low <= high {
            let mid = (unwrapped_low + high) / 2;
            match dataset[mid].get_title().cmp(&title) {
                Ordering::Equal => {
                    //return Some(mid);
                    return None;
                }
                Ordering::Less => {
                    unwrapped_low = mid + 1;
                }
                Ordering::Greater => {
                    high = mid - 1;
                }
            }
        }
        Some(unwrapped_low)
    } else {
        Some(dataset.len())
    }
}

/// Finds a chunk in the dataset that starts with the same letter as the key, returning the found index
/// One this chunk is found we binary search within that section, theoretically faster
///
/// # Arguments
///
/// * `dataset` - A dataset sorted in alphabetical order
/// * `key` - The key to look for
///
pub fn chunk_binary_search_index_load(dataset: &[Book], key: &String) -> Option<usize> {
    let title = key.to_string();
    //handel lower case
    let low = dataset
        .iter()
        .position(|b| b.get_title()[..1] == title[..1]);

    if let Some(index) = low {
        let mut high = dataset
            .iter()
            .rposition(|b| b.get_title()[..1] == title[..1])
            .unwrap();
        let mut unwrapped_low = index;
        while unwrapped_low <= high {
            let mid = (unwrapped_low + high) / 2;
            match dataset[mid].get_title().cmp(&title) {
                Ordering::Equal => {
                    return Some(mid);
                }
                Ordering::Less => {
                    unwrapped_low = mid + 1;
                }
                Ordering::Greater => {
                    high = mid - 1;
                }
            }
        }
        Some(unwrapped_low)
    } else {
        None
    }
}

/// Looks in an Epubs resources for a given key with a given mime type, returning the data as a string
///
/// # Arguments
///
/// * `key_regex` - The key to look for as regex query
/// * `mime_regex` - The mime type the key should have as a regex query
/// * `epub_resources` - The epubs resources to look in
/// * `doc` - A EpubDoc object to help with getting the resources
///
pub fn check_epub_resource(
    key_regex: Regex,
    mime_regex: Regex,
    epub_resources: &HashMap<String, (std::path::PathBuf, String)>,
    doc: &mut EpubDoc<BufReader<File>>,
) -> Option<String> {
    epub_resources
        .keys()
        .find(|key| {
            key_regex.is_match(key) && mime_regex.is_match(&doc.get_resource(key).unwrap().1)
        })
        .map(|key| key.to_owned())
}

pub fn create_batch_query(batch_books: Vec<&Book>) -> Result<String, ()> {
    let mut query_builder: sqlx::QueryBuilder<Sqlite> =
        sqlx::QueryBuilder::new("INSERT INTO books (cover_location, book_location, title) ");

    //TODO Might hit bind limits if users 'accumulates' books
    query_builder.push_values(batch_books.iter(), |mut b, book| {
        b.push_bind(book.get_cover_location())
            .push_bind(book.get_book_location())
            .push_bind(book.get_title());
    });

    let query = query_builder.into_sql();
    println!("{:?}", query);
    Ok(query)
}

pub fn get_cover_dir() -> PathBuf {
    let mut cache_dir = app_cache_dir(&current_context()).expect("Failed to get cache directory");
    cache_dir.push("cache");
    cache_dir.push(env!("COVER_IMAGE_FOLDER_NAME"));
    if let Err(err) = create_dir_all(&cache_dir) {
        eprintln!("Error creating {:?} directory: {:?}", err, cache_dir);
    }

    cache_dir
}
pub fn is_file_empty<P: AsRef<Path>>(file_path: P) -> bool {
    match fs::metadata(&file_path) {
        Ok(metadata) => metadata.len() == 0,
        Err(_) => {
            println!(
                "Failed to retrieve metadata for file: {}",
                file_path.as_ref().display()
            );
            false
        }
    }
}

/// Gets the current tauri context.
pub fn current_context() -> Config {
    generate_context!().config().clone()
}
