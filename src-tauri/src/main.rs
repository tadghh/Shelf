#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use app::{
    book::{
        bookio::{ __cmd__initialize_books, initialize_books },
        util::{
            __cmd__base64_encode_file,
            __cmd__base64_encode_covers,
            base64_encode_file,
            base64_encode_covers,
        },
        __cmd__load_book,
        load_book,
    },
    shelf::{
        __cmd__get_configuration_option,
        __cmd__change_configuration_option,
        __cmd__shelf_settings_values,
        change_configuration_option,
        get_configuration_option,
        shelf_settings_values,
    },
};

fn main() {
    tauri::Builder
        ::default()
        .invoke_handler(
            tauri::generate_handler![
                initialize_books,
                base64_encode_file,
                load_book,
                change_configuration_option,
                get_configuration_option,
                shelf_settings_values,
                base64_encode_covers
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
