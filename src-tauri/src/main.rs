#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
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
