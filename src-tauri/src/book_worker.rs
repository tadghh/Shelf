use std::{collections::HashMap, path::PathBuf};

use crate::{
    book::util::get_config_dir,
    book_item::{Book, BookCache},
};
pub struct BookWorker {
    cache_file_name: String,
    settings_file_name: String,
    cover_image_folder_name: String,
    config_folder_name: String,
    application_user_settings: HashMap<String, String>,
    current_book_cache: BookCache,
}
impl BookWorker {
    pub fn new(
        cache_file_name: String,
        settings_file_name: String,
        cover_image_folder_name: String,
        config_folder_name: String,
        application_user_settings: HashMap<String, String>,
        current_book_cache: BookCache,
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
    fn get_settings_file_name(&self) -> &String {
        &self.settings_file_name
    }
    pub fn get_settings_path(&self) -> PathBuf {
        get_config_dir().join(self.get_settings_file_name())
    }
    pub fn get_cache_file_name(&self) -> &String {
        &self.cache_file_name
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
        &self.current_book_cache
    }
    pub fn get_book_cache_test(&self) -> &BookCache {
        &self.current_book_cache
    }
    pub fn update_book_cache(&mut self, new_books: Vec<Book>) {
        self.current_book_cache.update_books(new_books)
    }
    pub fn update_cache_json_path(&mut self, json_path: String) {
        self.current_book_cache.update_json_path(json_path)
    }
}
