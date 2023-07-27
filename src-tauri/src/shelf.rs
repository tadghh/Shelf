use std::{collections::HashMap, fs::OpenOptions};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};


static CACHE_FILE_NAME: &str = "book_cache.json";
static SETTINGS_FILE_NAME: &str = "shelf_settings.conf";
static COVER_IMAGE_FOLDER_NAME: &str = "cover_cache";
pub fn get_cover_image_folder_name() -> &'static str{
     COVER_IMAGE_FOLDER_NAME
}
pub fn get_cache_file_name() -> &'static str{
    CACHE_FILE_NAME
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
#[tauri::command(rename_all = "snake_case")]
pub fn change_configuration_option(option_name: String, value: String) {
    let home_dir = &env::current_dir()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&settings_path)
        .unwrap();

    let mut lines = Vec::new();
    let mut updated = false;

    let reader = BufReader::new(&file);

    for line in reader.lines() {
        let line_content = line.unwrap();
        // println!("{:?} bull", &line_content);
        if line_content.starts_with(&option_name) {
            let updated_line = format!("{}={}", option_name, value);
            lines.push(updated_line);
            updated = true;
        } else {
            lines.push(line_content);
        }
    }

    if !updated {
        let new_line = format!("{}={}", option_name, value);
        lines.push(new_line);
    }

    let new_contents = lines.join("\n");
    let new_length = new_contents.len() as u64;

    file.seek(SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();
    file.write_all(new_contents.as_bytes()).unwrap();
    file.set_len(new_length).unwrap();
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_configuration_option(option_name: String) -> Option<String> {
    //todo: when program first runs we should look for files like this and create them
    let home_dir = &env::current_dir()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
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
            let split: Vec<&str> = line_content.split('=').collect();

            //Settings option not set
            if split[1] == "" {
                return None;
            }
            //"Valid" Option
            return Some(split[1].to_string());
        }
    }
    //Settings option missing
    None
}
