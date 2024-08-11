use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{
    book::util::{get_cache_dir, get_config_dir},
    book_item::{Book, BookCache},
    shelf::shelf_settings_values,
};
pub struct BookWorker {
    cache_file_name: String,
    settings_file_name: String,
    cover_image_folder_name: String,
    config_folder_name: String,
    application_user_settings: HashMap<String, String>,
    current_book_cache: Option<BookCache>,
}
impl BookWorker {
    pub fn new(
        cache_file_name: String,
        settings_file_name: String,
        cover_image_folder_name: String,
        config_folder_name: String,
        application_user_settings: HashMap<String, String>,
        current_book_cache: Option<BookCache>,
    ) -> BookWorker {
        BookWorker {
            cache_file_name,
            settings_file_name,
            cover_image_folder_name,
            config_folder_name,
            application_user_settings,
            current_book_cache,
        }
    }
    pub fn set_book_cache(&mut self, new_book_cache: BookCache) {
        self.current_book_cache = Some(new_book_cache)
    }
    fn get_settings_file_name(&self) -> &String {
        &self.settings_file_name
    }
    pub fn get_settings_path(&self) -> PathBuf {
        get_config_dir().join(self.get_settings_file_name())
    }
    pub fn get_cache_file_name(&self) -> &String {
        &self.cache_file_name
    }
    pub fn get_json_path(&self) -> String {
        get_cache_dir()
            .join(self.get_cache_file_name())
            .to_string_lossy()
            .to_string()
    }
    pub fn get_config_folder_name(&self) -> &String {
        &self.config_folder_name
    }
    pub fn get_cover_image_folder_name(&self) -> &String {
        &self.cover_image_folder_name
    }
    pub fn get_application_settings(&self) -> &HashMap<String, String> {
        &self.application_user_settings
    }

    pub fn get_book_cache(&self) -> &BookCache {
        let pecker = self.current_book_cache.as_ref().unwrap();
        &pecker
    }
    pub fn get_book_cache_test(&self) -> &BookCache {
        let pecker = self.current_book_cache.as_ref().unwrap();
        &pecker
    }
    pub fn update_book_cache(&mut self, new_books: Vec<Book>) {
        let pecker = self.current_book_cache.as_mut().unwrap();
        pecker.update_books(new_books);
    }

    pub fn update_cache_json_path(&mut self, json_path: String) {
        let pecker = self.current_book_cache.as_mut().unwrap();
        pecker.update_json_path(json_path)
    }
    pub fn restore_default_settings(&mut self) {
        let default_settings = shelf_settings_values();

        for (_setting_name, (lowercase_name, default_value)) in default_settings.iter() {
            self.update_application_setting(lowercase_name.to_string(), default_value.to_string());
        }
    }
    pub fn update_application_setting(&mut self, option_name: String, value: String) {
        self.application_user_settings
            .insert(option_name.clone(), value.clone());

        let settings_path = self.get_settings_path();
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
