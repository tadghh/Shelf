use std::{
    collections::HashMap,
    fs::{remove_dir_all, remove_file},
    sync::Mutex,
};

use tauri::State;

use crate::book_worker::{get_cache_dir, get_settings_path, load_settings, BookWorker};

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

/// To force overwrite users settings in memory

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
    let mut book_worker = state.lock().unwrap();

    //TODO: Handle these errors on front end, let user know it didnt work
    //Delete book json and covers
    println!("{:?}", get_settings_path());
    println!("{:?}", book_worker.get_cover_image_directory());
    //let mut book_worker = state.lock().unwrap();
    let cache_dir = get_cache_dir();
    let _ = remove_dir_all(cache_dir);

    //Delete settings file
    //If its an error thats okay because we remake the settings file anyway
    println!("{:?}", get_settings_path());
    let _ = remove_file(get_settings_path());

    //call default settings
    book_worker.restore_default_settings();
    book_worker.import_application_settings(load_settings());

    Ok(())
}
