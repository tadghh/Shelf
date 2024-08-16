#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::Mutex;

use app::*;

use app::book::bookio::initialize_books;
use app::{
    book_item::{get_cover_location_command, load_book},
    shelf::{
        change_configuration_option, get_configuration_option, reset_configuration,
        shelf_settings_values,
    },
};
use book_item::{get_all_books, BookCache};
use book_worker::{load_settings, BookWorker};
use database::import_book_json;
use tokio::runtime::Runtime;

fn main() {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    // Block on the async function `init_db`
    runtime.block_on(async {
        database::init_db().await;
    });

    // Now we can import a backup file if it exists
    _ = import_book_json();
    let current_books = get_all_books().ok();

    let mut worker = BookWorker::new(load_settings(), BookCache::new(current_books));

    let book_cache = BookCache::new(worker.initialize_books());

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
            reset_configuration,
            get_cover_location_command
        ])
        .run(tauri::generate_context!())
        .expect("shelf seems to have fallen over");
}
