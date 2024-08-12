#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::Mutex;

use app::*;

use app::book::bookio::initialize_books;
use app::{
    book_item::load_book,
    shelf::{
        change_configuration_option, get_configuration_option, reset_configuration,
        shelf_settings_values,
    },
};
use book_item::BookCache;
use book_worker::{load_settings, BookWorker};
fn main() {
    let mut worker = BookWorker::new(load_settings(), None);

    let book_cache = BookCache::new(worker.initialize_books(), worker.get_json_path());

    worker.set_book_cache(book_cache);

    let worker_mutex = Mutex::new(worker);

    tauri::Builder::default()
        .manage(worker_mutex)
        .invoke_handler(tauri::generate_handler![
            initialize_books,
            load_book,
            change_configuration_option,
            get_configuration_option,
            shelf_settings_values,
            reset_configuration
        ])
        .run(tauri::generate_context!())
        .expect("shelf seems to have fallen over");
}
