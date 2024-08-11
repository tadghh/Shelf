#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::Mutex;

use app::*;

use app::book::bookio::initialize_books;
use app::{
    book::{bookio::initialize_books_start, util::base64_encode_file},
    book_item::load_book,
    shelf::{
        change_configuration_option, get_configuration_option, reset_configuration,
        shelf_settings_values,
    },
};
use book::util::get_cache_dir;
use book_item::BookCache;
use book_worker::BookWorker;
use shelf::load_settings;
fn main() {
    let cache_name = env!("CACHE_F_NAME");
    let settings_name = env!("SETTINGS_F_NAME");
    let config_folder_name = env!("CONFIG_FLDR_NAME");
    let cover_folder_name = env!("COVER_IMAGE_FOLDER_NAME");

    let mut worker = BookWorker::new(
        cache_name.to_owned(),
        settings_name.to_owned(),
        cover_folder_name.to_owned(),
        config_folder_name.to_owned(),
        load_settings(),
        None,
    );

    let book_location = worker
        .get_application_settings()
        .get("book_location")
        .unwrap();

    let json_path = get_cache_dir()
        .join(env!("CACHE_F_NAME"))
        .to_string_lossy()
        .to_string();

    let book_cache = BookCache::new(initialize_books_start(
        cover_folder_name.to_string(),
        json_path,
        book_location,
    ));

    worker.set_book_cache(book_cache);

    let worker_mutex = Mutex::new(worker);

    tauri::Builder::default()
        .manage(worker_mutex)
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
