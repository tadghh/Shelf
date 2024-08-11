use std::{
    collections::HashMap,
    fs::{remove_dir_all, remove_file, OpenOptions},
    io::{BufRead, BufReader, Error, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use tauri::State;

use crate::{
    book::util::{get_cache_dir, get_config_dir},
    book_worker::BookWorker,
};

fn get_settings_path() -> PathBuf {
    get_config_dir().join(env!("SETTINGS_F_NAME"))
}
fn get_covers_path() -> PathBuf {
    get_cache_dir().join(env!("COVER_IMAGE_FOLDER_NAME"))
}

///This is how we get out settings back over to nextjs.
///TODO: Use enums throughout backend, lazy guy :|
/// TODO Why is this a command its only useful from the backend
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
        .map(|(k, v)| (k.clone(), (k.to_lowercase(), v.to_string())))
        .collect()
}

/// Creates a settings file and fills it with mostly valid default values.
fn create_default_settings() -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(get_settings_path())
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
pub fn load_settings() -> HashMap<String, String> {
    let settings_path = get_settings_path();
    let bro = Path::new(&settings_path);
    if !bro.exists() {
        let _ = create_default_settings();
    }
    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&settings_path)
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

        if split.len() == 2 {
            settings_map.insert(split[0].to_string(), split[1].to_string());
        }
    }

    settings_map
}

///Load user settings into memory, if they havent already been

/// Sets all settings consts to be "unset" or default

/// Returns the setting for the provided value
///
/// # Arguments
///
/// * `option_name` - The setting to get the value of
///
#[tauri::command(rename_all = "snake_case")]
pub fn get_configuration_option(
    option_name: String,
    state: State<'_, Mutex<BookWorker>>,
) -> Option<String> {
    // load_settings_into_memory();
    let book_worker = state.lock().unwrap();
    let application_settings = book_worker.get_application_settings().clone();

    let settings_value = application_settings.get(&option_name);
    // let settings_value = unsafe {
    //     SETTINGS_MAP
    //         .as_ref()
    //         .and_then(|map| map.get(&option_name).cloned())
    // };

    settings_value.cloned()
}

#[allow(static_mut_refs)]
/// Changes the value of a settings item
///
/// # Arguments
///
/// * `option_name` - The setting to change
/// * `value` - The new value to set
///
/// #[warn(static_mut_refs)]

#[tauri::command(rename_all = "snake_case")]
pub fn change_configuration_option(
    option_name: String,
    value: String,
    state: State<'_, Mutex<BookWorker>>,
) {
    let mut book_worker = state.lock().unwrap();
    book_worker.update_application_setting(option_name, value);
}

//Delete config files and call the create file method
#[tauri::command(rename_all = "snake_case")]
pub fn reset_configuration(state: State<'_, Mutex<BookWorker>>) -> Result<(), String> {
    //TODO: Handle these errors on front end, let user know it didnt work
    //Delete book json and covers
    println!("{:?}", get_settings_path());
    println!("{:?}", get_covers_path());
    let mut book_worker = state.lock().unwrap();

    let _ = remove_dir_all(get_cache_dir());

    //Delete settings file
    //If its an error thats okay because we remake the settings file anyway
    println!("{:?}", get_settings_path());
    let _ = remove_file(get_settings_path());

    //call default settings
    book_worker.restore_default_settings();
    load_settings();

    Ok(())
}
