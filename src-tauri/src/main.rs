#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
mod book;
mod shelf;
mod xml;
use crate::{
    book::{
        load_book,
        get_cover,
        util::{ base64_encode_file, base64_encode_covers },
        bookio::create_covers,
    },
    shelf::{ change_configuration_option, get_configuration_option, shelf_settings_values },
};

fn main() {
    tauri::Builder
        ::default()
        .invoke_handler(
            tauri::generate_handler![
                create_covers,
                base64_encode_file,
                load_book,
                change_configuration_option,
                get_configuration_option,
                get_cover,
                shelf_settings_values,
                base64_encode_covers
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
