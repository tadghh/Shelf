use std::{collections::HashMap, sync::Mutex};

use tauri::State;

use crate::book_worker::BookWorker;

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
    let book_worker = state.lock().unwrap();
    let application_settings = book_worker.get_application_settings();

    let settings_value = application_settings.get(&option_name);

    settings_value.cloned()
}

/// Changes the value of a settings item
///
/// # Arguments
///
/// * `option_name` - The setting to change
/// * `value` - The new value to set
///
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
    // TODO use array for errors
    book_worker.reset();

    Ok(())
}
