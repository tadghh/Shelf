use std::{
    collections::HashMap,
    fs::{OpenOptions, remove_file, remove_dir_all},
    io::{ BufRead, BufReader, Seek, SeekFrom, Write, Read },
    path::PathBuf,
};

use crate::book::util::{get_config_dir, get_cache_dir};

static CACHE_FILE_NAME: &str = "book_cache.json";
static SETTINGS_FILE_NAME: &str = "shelf_settings.conf";
static COVER_IMAGE_FOLDER_NAME: &str = "cover_cache";
static mut SETTINGS_MAP: Option<HashMap<String, String>> = None;

fn get_settings_path() -> PathBuf {
    get_config_dir().join(SETTINGS_FILE_NAME)
}

///Get the name of the cover image folder
pub fn get_cover_image_folder_name() -> &'static str {
    COVER_IMAGE_FOLDER_NAME
}

///Get the book cache file name
pub fn get_cache_file_name() -> &'static str {
    CACHE_FILE_NAME
}

///Get the name of the settings file
pub fn get_settings_name() -> &'static str {
    SETTINGS_FILE_NAME
}

///This is how we get out settings back over to nextjs.
///TODO: Use enums throughout backend, lazy guy :| 
#[tauri::command]
pub fn shelf_settings_values() -> HashMap<String, String> {
    let setting_consts = ["BOOK_LOCATION","ENDLESS_SCROLL","COVER_BACKGROUND"];

    let shelf_option_values: HashMap<String, String> = setting_consts
    .iter()
    .map(|entry| (entry.to_string(), entry.to_lowercase()))
    .collect();

    shelf_option_values
}

/// To force overwrite users settings in memory
fn load_settings(){
    let settings_path = get_settings_path();

            let file = match
                OpenOptions::new().read(true).write(true).create(true).open(&settings_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error opening settings file, trying to create one: {}", e);
                    restore_default_settings();
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

            unsafe { SETTINGS_MAP = Some(settings_map) };
}

///Load user settings into memory
fn load_settings_into_memory() {
    unsafe {
        if SETTINGS_MAP.is_none() {
            //let home_dir = get_home_dir();
            // let settings_path = format!("{}/{}", home_dir, &SETTINGS_FILE_NAME);
            let settings_path = get_settings_path();
            // Check if the file already exists
            let file = match
                OpenOptions::new().read(true).write(true).create(true).open(&settings_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error opening settings file, trying to create one: {}", e);
                    restore_default_settings();
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

/// Sets all settings consts to be "unset" or default
///
pub fn restore_default_settings() {
    load_settings_into_memory();

    //TODO: Unset is not a "good" default value
    for entry in shelf_settings_values().iter() {
        change_configuration_option(entry.1.to_owned(), "Unset".to_string());
    }
}

/// Returns the setting for the provided value
///
/// # Arguments
///
/// * `option_name` - The setting to get the value of
///
#[tauri::command(rename_all = "snake_case")]
pub fn get_configuration_option(option_name: String) -> Option<String> {
    load_settings_into_memory();

    //TODO: Could encounter error if memory issue
    let value = unsafe { SETTINGS_MAP.as_ref().and_then(|map| map.get(&option_name).cloned()) };

    value
}

/// Changes the value of a settings item
///
/// # Arguments
///
/// * `option_name` - The setting to change
/// * `value` - The new value to set
///
///
#[tauri::command(rename_all = "snake_case")]
pub fn change_configuration_option(option_name: String, value: String) {
    load_settings_into_memory();
    unsafe {
        if let Some(map) = &mut SETTINGS_MAP {
            map.insert(option_name.clone(), value.clone());

            let settings_path = get_settings_path();
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(settings_path)
                .unwrap();

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            if let Some(index) = contents.find(&format!("{}=", option_name)) {
                let start = index + option_name.len() + 1;

                if let Some(end) = contents[start..].find('\n') {
                    // Option found with a newline character after
                    let mut new_contents = contents.clone();
                    let new_value = value;
                    new_contents.replace_range(start..start + end, &new_value);

                    // Overwrite the file with the updated contents
                    file.seek(SeekFrom::Start(0)).unwrap();
                    file.set_len(0).unwrap();
                    file.write_all(new_contents.as_bytes()).unwrap();
                } else {
                    // Option found without a newline character after
                    let new_value = format!("{}\n", value);
                    contents.push_str(&new_value);

                    // Append the new line to the end of the file
                    file.seek(SeekFrom::End(0)).unwrap();
                    file.write_all(new_value.as_bytes()).unwrap();
                }
            } else {
                // Option not found, so add it with a newline character after
                let new_line = format!("{}={}\n", option_name, value);
                contents.push_str(&new_line);

                // Append the new line to the end of the file
                file.seek(SeekFrom::End(0)).unwrap();
                file.write_all(new_line.as_bytes()).unwrap();
            }
        }
    }
}


//Delete config files and call the create file method
#[tauri::command(rename_all = "snake_case")]
pub fn reset_configuration() -> Result<(),  String>{

    //Delete book json and covers
    if let Err(err) = remove_dir_all(get_cache_dir()) {
        return Err(err.to_string());
    }

    //Delete settings file
    if let Err(err) = remove_file(get_settings_path()) {
        return Err(err.to_string());
    }
    //call default settings
    restore_default_settings();
    load_settings();

    Ok(())
}


