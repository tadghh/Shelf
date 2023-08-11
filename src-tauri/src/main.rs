#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use app::{ book::{ bookio::*, util::*, * }, shelf::* };

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
