use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::OpenOptions,
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
};

use crate::book::bookio::{create_default_settings_file, get_home_dir};

static mut SETTINGS_MAP: Option<HashMap<String, String>> = None;

static CACHE_FILE_NAME: &str = "book_cache.json";
static SETTINGS_FILE_NAME: &str = "shelf_settings.conf";
static COVER_IMAGE_FOLDER_NAME: &str = "cover_cache";
pub fn get_cover_image_folder_name() -> &'static str {
    COVER_IMAGE_FOLDER_NAME
}
pub fn get_cache_file_name() -> &'static str {
    CACHE_FILE_NAME
}
pub fn get_settings_name() -> &'static str {
    SETTINGS_FILE_NAME
}
#[derive(Serialize, Deserialize, Debug)]
enum Settings {
    EndlessScroll,
    BookLocation,
}
#[tauri::command]
pub fn shelf_settings_health() -> HashMap<String, String> {
    // this is making me sad
    //I have a list of keys I will know exist
    // I know what type these keys should be
    enum ValueItem {
        String(String),
        Bool(bool),
        Float(f64),
    }
    struct SettingsItem {
        item_key: String,
        default_value: ValueItem,
    }
    let expected_keys = vec![
        SettingsItem {
            item_key: "book_folder_location".to_string(),
            default_value: ValueItem::String("E:\\Books\\Book\\Epub".to_string()),
        },
        SettingsItem {
            item_key: "endless_scroll".to_string(),
            default_value: ValueItem::Bool(false),
        },
    ];
    //check if settings file exists
    // Oh theres a file? lets verify the values
    // map over the file comparing agaisnt expected keys
    // if the value is good leave it otherwise use the default
    //if not create it
    let mut map = HashMap::new();
    map.insert(
        String::from("ENDLESS_SCROLL"),
        String::from("endless_scroll"),
    );
    map.insert(
        String::from("BOOK_LOCATION"),
        String::from("book_folder_location"),
    );
    map
    // loop through list of settings and itialize default key and vals
}
#[tauri::command]
fn shelf_setup() {
    //file name to long
    shelf_settings_health();
    //if it does load it?
}

fn load_settings_into_memory() {
    unsafe {
        if SETTINGS_MAP.is_none() {
            let home_dir = get_home_dir();
            let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
            // Check if the file already exists
            let file = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&settings_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error opening settings file, trying to create one: {}", e);
                    create_default_settings_file();
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&settings_path)
                        .expect("Failed to open settings file")
                }
            };

            let reader = BufReader::new(&file);

            let mut settings_map = HashMap::new();
            for line in reader.lines() {
                let line_content = line.unwrap();
                let split: Vec<&str> = line_content.split('=').collect();
                if split.len() == 2 {
                    settings_map.insert(split[0].to_string(), split[1].to_string());
                }
            }

            SETTINGS_MAP = Some(settings_map);
        }
    }
}
#[tauri::command(rename_all = "snake_case")]
pub fn get_configuration_option(option_name: String) -> Option<String> {
    load_settings_into_memory();
    unsafe {
        SETTINGS_MAP
            .as_ref()
            .and_then(|map| map.get(&option_name).cloned())
    }
}
#[tauri::command(rename_all = "snake_case")]
pub fn change_configuration_option(option_name: String, value: String) {
    load_settings_into_memory();
    unsafe {
        if let Some(map) = &mut SETTINGS_MAP {
            map.insert(option_name.clone(), value.clone());

            let home_dir = get_home_dir();
            let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&settings_path)
                .unwrap();

            let mut lines = Vec::new();
            for (key, val) in map.iter() {
                let line = format!("{}={}", key, val);
                lines.push(line);
            }

            let new_contents = lines.join("\n");
            let new_length = new_contents.len() as u64;

            file.seek(SeekFrom::Start(0)).unwrap();
            file.set_len(0).unwrap();
            file.write_all(new_contents.as_bytes()).unwrap();
            file.set_len(new_length).unwrap();
        }
    }
}
