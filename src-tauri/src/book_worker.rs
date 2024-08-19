use std::{
    collections::HashMap,
    fs::{self, create_dir_all, remove_dir_all, remove_file, File, OpenOptions},
    io::{BufReader, Error, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use tauri::{
    api::path::{app_cache_dir, app_config_dir},
    State,
};

use crate::{
    book::{bookio::create_book_vec, util::current_context},
    book_item::{
        create_books_table, drop_books_from_table, get_all_books, insert_book_db_batch, Book,
        BookCache,
    },
    database::{append_date_to_filename, check_db_health, import_book_json},
    shelf::shelf_settings_values,
};

// This worker object allowed me to replace the global statics I was using before.
// We leverage tauris manage state feature to access it when needed
pub struct BookWorker {
    application_user_settings: HashMap<String, String>,
    current_book_cache: BookCache,
}
impl BookWorker {
    pub fn new(
        application_user_settings: HashMap<String, String>,
        current_book_cache: BookCache,
    ) -> BookWorker {
        BookWorker {
            application_user_settings,
            current_book_cache,
        }
    }

    pub fn set_book_cache(&mut self, new_book_cache: BookCache) {
        self.current_book_cache = new_book_cache
    }

    pub fn get_book_cache(&self) -> &BookCache {
        &self.current_book_cache
    }

    pub fn reset(&mut self) {
        _ = remove_dir_all(get_cache_dir());

        //Delete settings file
        //If its an error thats okay because we remake the settings file anyway
        _ = remove_file(get_settings_path());
        _ = drop_books_from_table();

        self.update_book_cache(None);
        self.restore_default_settings();
        _ = create_books_table();
    }

    pub fn import_application_settings(&mut self, new_book_cache: HashMap<String, String>) {
        self.application_user_settings = new_book_cache
    }

    pub fn backup_current_books(&mut self, write_dir: Option<String>) {
        let json_dump_path = match write_dir {
            Some(path) => {
                let mut export_file_name = PathBuf::from(path);
                export_file_name = export_file_name.join("export.json");

                // is returned a string here
                // export_file_name =

                match export_file_name.to_str() {
                    Some(valid_str) => Some(PathBuf::from(append_date_to_filename(valid_str))),
                    None => None,
                }
            }
            None => get_dump_json_path(),
        };
        match &self.get_book_cache().get_books() {
            Some(all_books) => match json_dump_path {
                Some(path) => {
                    let file = File::create(path)
                        .expect("JSON backup path should be defined, and a valid json file");

                    serde_json::to_writer(file, &all_books)
                        .expect("failed to write to backup json file!");
                }
                None => println!("Failed to make json dump file"),
            },
            None => match get_all_books() {
                Ok(db_books) => match json_dump_path {
                    Some(path) => {
                        let file = File::create(path)
                            .expect("JSON backup path should be defined, and a valid json file");

                        serde_json::to_writer(file, &db_books)
                            .expect("failed to write to backup json file!");
                    }
                    None => println!("Failed to make json dump file"),
                },
                Err(_) => println!("Failed to create backup, no books in memory or the database"),
            },
        }
    }

    // check if db file is missing
    // run backup current books if it is
    // run import method
    pub fn repair_db(&mut self) {
        if !check_db_health() {
            self.backup_current_books(None);

            _ = import_book_json(None);
        }
    }

    // TODO support multiple book location
    pub fn get_application_settings(&self) -> &HashMap<String, String> {
        &self.application_user_settings
    }

    // Updates the book objects items
    fn update_books(&mut self, new_books: Vec<Book>) {
        let current_books = get_all_books()
            .ok()
            .or_else(|| self.get_book_cache().get_books().cloned());

        let unique_new_books: Vec<_> = new_books
            .into_iter()
            .filter(|book| {
                current_books
                    .as_ref()
                    .map_or(true, |books| !books.contains(book))
            })
            .collect();

        if unique_new_books.is_empty() {
            println!("No new unique books found or current_books was None");
            println!("epub length different but no new books");
            return;
        }

        let mut all_books = self
            .get_book_cache()
            .get_books()
            .cloned()
            .unwrap_or_default();
        all_books.extend(unique_new_books.clone());

        // Update book contents in memory
        self.update_book_cache(Some(all_books));

        // try to update local storage book contents
        if insert_book_db_batch(&unique_new_books).is_err() {
            println!("Failed to update books, dumping to backup file");
            self.repair_db();
        }
    }

    // concat method
    pub fn update_book_cache(&mut self, new_books: Option<Vec<Book>>) {
        self.current_book_cache.update_books(new_books)
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

    pub fn initialize_books(&mut self) -> Option<Vec<Book>> {
        let dir = self.get_application_settings().get("book_location")?;

        if !Path::new(&dir).exists() {
            return None;
        }

        //yes you could break this, but im not being paid
        // bug: swap out a already processed book with a new one
        let epub_paths: Vec<String> = fs::read_dir(dir)
            .ok()?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension()? == "epub" {
                    path.to_str().map(|s| s.to_owned())
                } else {
                    None
                }
            })
            .collect();

        if self.get_book_cache().get_book_amount() != epub_paths.len() {
            self.update_books(create_book_vec(&epub_paths));
        }

        self.get_book_cache().get_books().cloned()
    }
}

// Functions that are related but need to be accessed elsewhere

#[tauri::command]
pub fn backup_books_to_json(path: String, state: State<'_, Mutex<BookWorker>>) {
    let mut book_worker = state.lock().unwrap();

    book_worker.backup_current_books(Some(path));
}

pub fn get_settings_path() -> PathBuf {
    get_config_dir().join(env!("SETTINGS_F_NAME"))
}

pub fn get_cache_dir() -> PathBuf {
    let mut cache_dir = app_cache_dir(&current_context()).expect("Failed to get cache directory");
    cache_dir.push("cache");
    if let Err(err) = create_dir_all(&cache_dir) {
        eprintln!("Error creating cache directory: {:?}", err);
    }

    cache_dir
}

pub fn get_dump_json_path() -> Option<PathBuf> {
    let path = get_cache_dir();
    // TODO json dump path failed to create
    _ = create_dir_all(get_cache_dir());
    Some(path.join(env!("BACKUP_FILENAME")))
}

pub fn load_settings() -> HashMap<String, String> {
    let settings_path = get_settings_path();

    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(&settings_path)
    {
        Ok(file) => file,
        Err(_) => {
            create_default_settings().expect("While loading the user settings an issue occurred. Resulting in the fallback defaults failing")
        }
    };

    let mut settings_map = HashMap::new();

    for line in std::io::BufRead::lines(BufReader::new(&file)) {
        let line_content = line.unwrap();
        let split: Vec<&str> = line_content.split('=').collect();

        if split.len() == 2 {
            settings_map.insert(split[0].to_string(), split[1].to_string());
        }
    }

    settings_map
}

/// Creates the settings file and sets default values
pub fn create_default_settings() -> Result<File, Error> {
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

    Ok(file)
}

pub fn get_cover_image_directory() -> Option<PathBuf> {
    let cover_path = get_cache_dir().join(env!("COVER_IMAGE_FOLDER_NAME"));
    create_dir_all(&cover_path).ok().map(|_| cover_path)
}

pub fn get_config_dir() -> PathBuf {
    let mut full_config_path =
        app_config_dir(&current_context()).expect("Failed to get config directory");
    full_config_path.push(env!("CONFIG_FLDR_NAME"));

    if let Err(err) = create_dir_all(&full_config_path) {
        eprintln!("Error creating config directory: {:?}", err);
    }

    full_config_path
}
