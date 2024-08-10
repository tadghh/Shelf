#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
<<<<<<< HEAD
use app::*;

use app::{
    book::{bookio::initialize_books, util::base64_encode_file},
    book_item::load_book,
    shelf::{
        change_configuration_option, get_configuration_option, reset_configuration,
        shelf_settings_values,
    },
};

fn main() {
    tauri::Builder::default()
=======
use std::collections::HashMap;

use app::book::{bookio::initialize_books, load_book, util::base64_encode_file};

use app::shelf::{
    change_configuration_option, get_configuration_option, reset_configuration,
    shelf_settings_values,
};

use app::*;
use book::Book;
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
struct BookWorker {
    cache_file_name: String,
    settings_file_name: String,
    cover_image_folder_name: String,
    config_file_name: String,
    application_user_settings: HashMap<String, String>,
    current_books: BookCache,
}
fn main() {
    tauri::Builder::default()
        .manage(state)
>>>>>>> 6e3d127 (added: worker along with comments)
        .invoke_handler(tauri::generate_handler![
            initialize_books,
            base64_encode_file,
            load_book,
            change_configuration_option,
            get_configuration_option,
            shelf_settings_values,
            reset_configuration
        ])
        .run(tauri::generate_context!())
        .expect("shelf seems to have fallen over");
}
