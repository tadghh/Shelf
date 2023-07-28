use serde::{ Deserialize, Serialize };
use std::{
    collections::HashMap,
    env,
    fs::{ self, OpenOptions },
    io::{ BufRead, BufReader, Seek, SeekFrom, Write, Read },
};

use crate::book::bookio::{ create_default_settings_file, get_home_dir };

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
#[tauri::command]
pub fn shelf_settings_health() -> HashMap<String, String> {
    let expected_keys: Vec<(String, String)> = vec![
        ("BOOK_LOCATION".to_string(), "book_folder_location".to_string()),
        ("ENDLESS_SCROLL".to_string(), "endless_scroll".to_string())
        // Add more settings names here as needed
    ];

    let mut settings_map = HashMap::new();

    for (key, name) in expected_keys {
        settings_map.insert(key, name);
    }

    settings_map
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_configuration_option(option_name: String) -> Option<String> {
    //todo: when program first runs we should look for files like this and create them
    let home_dir = get_home_dir();
    let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
    println!("yo {:?}", &settings_path);
    println!("yo {:?}", &option_name);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&settings_path)
        .expect("The settings file should exist");

    let reader = BufReader::new(&file);

    for line in reader.lines() {
        let line_content = line.unwrap();

        if line_content.starts_with(&option_name) {
            let split: Vec<&str> = line_content.split("=").collect();
            println!(" early");
            return Some(split[1].to_string());
        }
    }
    println!(" early");

    change_configuration_option(option_name, "yo".to_string());
    return Some("String".to_string());
}

#[tauri::command(rename_all = "snake_case")]
pub fn change_configuration_option(option_name: String, value: String) {
    let home_dir = get_home_dir();
    let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&settings_path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    if let Some(index) = contents.find(&format!("{}=", option_name)) {
        let start = index + option_name.len() + 1;

        if let Some(end) = contents[start..].find('\n') {
            // Option found with a newline character after
            let new_value = format!("{}", value);

            let mut new_contents = String::with_capacity(contents.len());
            new_contents.push_str(&contents[..start]);
            new_contents.push_str(&new_value);
            new_contents.push_str(&contents[start + end..]);
            file.seek(SeekFrom::Start(0)).unwrap();
            file.set_len(0).unwrap();
            file.write_all(new_contents.as_bytes()).unwrap();
        } else {
            // Option found without a newline character after
            let new_value = format!("{}\n", value);

            let mut new_contents = String::with_capacity(contents.len());
            new_contents.push_str(&contents[..start]);
            new_contents.push_str(&new_value);
            file.seek(SeekFrom::Start(0)).unwrap();
            file.set_len(0).unwrap();
            file.write_all(new_contents.as_bytes()).unwrap();
        }
    }
}
