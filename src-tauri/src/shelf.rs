use std::{
    collections::HashMap,
    fs::{OpenOptions, remove_file, remove_dir_all, File},
    io::{ BufRead, BufReader, Seek, SeekFrom, Write, Read },
    path::{PathBuf, Path},
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

///This is how we get out settings back over to nextjs.
///TODO: Use enums throughout backend, lazy guy :|
#[tauri::command]
pub fn shelf_settings_values() -> HashMap<String, (String, String)> {
    let setting_consts: HashMap<String, &str> = [
        ("BOOK_LOCATION".to_string(), "unset"),
        ("ENDLESS_SCROLL".to_string(), "false"),
        ("COVER_BACKGROUND".to_string(), "false"),
    ]
    .iter()
    .cloned()
    .collect();

    setting_consts
        .into_iter()
        .map(|(k, v)| {
            (
                k.clone(),
                (k.to_lowercase(), v.to_string()),
            )
        })
        .collect()
}

/// Creates a settings file and fills it with mostly valid default values.
///
///  * `settings_path` - The path to the settings file
///
fn create_default_settings(settings_path: &Path) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(settings_path)
        .expect("Failed to open or create settings file");

    // Generate default settings from shelf_settings_values
    let default_settings: HashMap<String, (String, String)> = shelf_settings_values();

    for (_setting_name, (lowercase_name, default_value)) in default_settings.iter() {
        let setting_str = format!("{}={}\n", lowercase_name, default_value);
        file.write_all(setting_str.as_bytes())?;
    }
    Ok(())
}
/// To force overwrite users settings in memory
fn load_settings(){
    let settings_path = get_settings_path();
    let bro = Path::new(&settings_path);
    if !bro.exists() {
        let _ = create_default_settings(&settings_path);
    }
    let file = match
        OpenOptions::new().read(true).write(true).create(true).open(&settings_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening settings file, trying to create one: {}", e);

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
        print!("{:?}", split);
        if split.len() == 2 {
            settings_map.insert(split[0].to_string(), split[1].to_string());
        }
    }

    unsafe { SETTINGS_MAP = Some(settings_map) };
}

///Load user settings into memory, if they havent already been
fn load_settings_into_memory() {
    unsafe {
        if SETTINGS_MAP.is_none() {
            load_settings();
        }
    }
}

/// Sets all settings consts to be "unset" or default
pub fn restore_default_settings() {
    load_settings_into_memory();

    //TODO: Unset is not a "good" default value
    let default_settings: HashMap<String, (String, String)> = shelf_settings_values();

    for (_setting_name, (lowercase_name, default_value)) in default_settings.iter() {
        change_configuration_option(lowercase_name.to_string(), default_value.to_string());

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
    let settings_value = unsafe { SETTINGS_MAP.as_ref().and_then(|map| map.get(&option_name).cloned()) };

    settings_value
}

/// Changes the value of a settings item
///
/// # Arguments
///
/// * `option_name` - The setting to change
/// * `value` - The new value to set
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

    //TODO: Handle these errors on front end, let user know it didnt work
    //Delete book json and covers
    if let Err(err) = remove_dir_all(get_cache_dir()) {
        return Err(err.to_string());
    }

    //Delete settings file
    //If its and error thats okay because we remake it anyway
    let _ = remove_file(get_settings_path());

    //call default settings
    restore_default_settings();
    load_settings();

    Ok(())
}


