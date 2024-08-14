use std::{
    collections::HashMap,
    fs::{self, create_dir_all, OpenOptions},
    io::{BufReader, Error, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use tauri::api::path::{app_cache_dir, app_config_dir};

use crate::{
    book::{bookio::create_book_vec, util::current_context},
    book_item::{get_all_books, insert_book_db_batch, Book, BookCache},
    shelf::shelf_settings_values,
};

// This worker object allowed me to replace the global statics I was using before.
// We leverage tauris manage state feature to access it when needed
pub struct BookWorker {
    application_user_settings: HashMap<String, String>,
    current_book_cache: Option<BookCache>,
}
impl BookWorker {
    pub fn new(
        application_user_settings: HashMap<String, String>,
        current_book_cache: Option<BookCache>,
    ) -> BookWorker {
        BookWorker {
            application_user_settings,
            current_book_cache,
        }
    }
    /// Creates a settings file and fills it with mostly valid default values.

    pub fn set_book_cache(&mut self, new_book_cache: BookCache) {
        self.current_book_cache = Some(new_book_cache)
    }
    pub fn import_application_settings(&mut self, new_book_cache: HashMap<String, String>) {
        self.application_user_settings = new_book_cache
    }

    // pub fn get_settings_path(&self) -> PathBuf {
    //     get_config_dir().join(self.get_settings_file_name())
    // }

    pub fn get_json_path(&self) -> String {
        self.get_cache_dir()
            .join(env!("CACHE_F_NAME"))
            .to_string_lossy()
            .to_string()
    }
    pub fn get_config_folder_name(&self) -> String {
        env!("CONFIG_FLDR_NAME").to_string()
    }

    pub fn get_cover_image_directory(&self) -> Option<PathBuf> {
        let covers_directory = self.get_covers_path();
        //covers_directory.push(self.get_cover_image_folder_name());
        match create_dir_all(covers_directory.clone()) {
            Ok(()) => Some(covers_directory),
            Err(_) => None,
        }
    }

    // TODO support multiple book location
    pub fn get_application_settings(&self) -> &HashMap<String, String> {
        &self.application_user_settings
    }

    pub fn get_book_cache(&self) -> &BookCache {
        let pecker = self.current_book_cache.as_ref().unwrap();
        &pecker
    }
    // pub fn get_book(&self) -> &BookCache {
    //     let pecker = self.current_book_cache.as_ref().unwrap();
    //     &pecker
    // }

    pub fn update_book_cache(&mut self, new_books: Vec<Book>) {
        let pecker = self.current_book_cache.as_mut().unwrap();
        pecker.update_books(new_books);
    }

    // pub fn update_cache_json_path(&mut self, json_path: String) {
    //     let pecker = self.current_book_cache.as_mut().unwrap();
    //     pecker.update_json_path(json_path)
    // }
    pub fn restore_default_settings(&mut self) {
        let default_settings = shelf_settings_values();

        for (_setting_name, (lowercase_name, default_value)) in default_settings.iter() {
            self.update_application_setting(lowercase_name.to_string(), default_value.to_string());
        }
    }
    fn get_cache_dir(&self) -> PathBuf {
        let mut cache_dir =
            app_cache_dir(&current_context()).expect("Failed to get cache directory");
        cache_dir.push("cache");
        if let Err(err) = create_dir_all(&cache_dir) {
            eprintln!("Error creating cache directory: {:?}", err);
        }

        cache_dir
    }
    pub fn get_covers_path(&self) -> PathBuf {
        self.get_cache_dir().join(env!("COVER_IMAGE_FOLDER_NAME"))
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
    pub fn initialize_books(&self) -> Option<Vec<Book>> {
        let start_time = Instant::now();

        //  let mut file_changes = false;

        let current_books: Vec<Book>;
        //let book_worker = state.lock().unwrap();

        // let json_path: String = self.get_json_path();

        let dir = match self.get_application_settings().get("book_location") {
            Some(val) => val,
            None => {
                return None;
            }
        };
        // TODO bug the db could still have books
        if !Path::new(&dir).exists() {
            return None;
        }

        let epub_paths: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                match path {
                    p if p.is_file() && p.extension()? == "epub" => {
                        Some(p.to_str().unwrap().to_owned())
                    }
                    _ => None,
                }
            })
            .collect();

        let epub_amount = epub_paths.len();

        // TODO make sure default is used if this is none (not in this exact context)

        // if Path::new(&json_path).exists() {
        // let file = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .create(true)
        //     .open(&json_path);
        let new_books = create_book_vec(&epub_paths);

        current_books = match get_all_books() {
            Ok(books) => books,
            Err(_) => todo!(),
        };

        // current_books = match serde_json::from_reader(BufReader::new(file.unwrap())) {
        //     Ok(data) => data,
        //     Err(_) => Vec::new(),
        // };

        let current_length = current_books.len();
        match current_length {
            0 => {
                let new_books_refs: Vec<&Book> = new_books.iter().map(|book| book).collect();
                match insert_book_db_batch(new_books_refs) {
                    Ok(_) => println!("Insert worked"),
                    Err(_) => println!("INsert not so work"),
                };
                println!("Execution time: {} ms", start_time.elapsed().as_millis());
                Some(new_books)
            }
            _ if current_length != epub_amount => {
                let unique_new_books: Vec<_> = new_books
                    .into_iter()
                    .filter(|book| !current_books.contains(book))
                    .collect();
                if unique_new_books.len() != 0 {
                    // handled lining up the data to 'skip' sorting it
                    // let mut index_offset = 0;
                    // for (book, index) in &new_books {
                    //     current_books.insert(index + index_offset, book.to_owned());
                    //     index_offset += 1;
                    // }
                    let new_books: Vec<&Book> = unique_new_books.iter().map(|book| book).collect();

                    match insert_book_db_batch(new_books) {
                        Ok(_) => println!("Insert worked"),
                        Err(_) => println!(
                            "INsert not so work maybe book with same title but diff author"
                        ),
                    };
                    println!("Execution time: {} ms", start_time.elapsed().as_millis());
                    return Some(unique_new_books);
                }
                println!("epub length different but no new books");
                println!("Execution time: {} ms", start_time.elapsed().as_millis());
                Some(current_books)
            }
            _ => {
                println!("no new books");
                println!("Execution time: {} ms", start_time.elapsed().as_millis());
                Some(current_books)
            }
        }
        // let epub_reader = EpubDoc::new
        // if current_length != epub_amount {
        // let new_books: Vec<(Book, usize)> = epub_paths
        //     .par_iter()
        //     .filter_map(|item| {
        //         let item_normalized = item.replace('\\', "/");

        //         match EpubDoc::new(&item_normalized) {
        //             Ok(ebook) => {
        //                 let book_title = ebook.mdata("title")?;
        //                 let index = chunk_binary_search_index(&current_books, &book_title)?;

        //                 let new_book = Book::new(None, item_normalized, book_title);

        //                 Some((new_book, index))
        //             }
        //             Err(e) => {
        //                 println!("Book creation failed with: {}", e);

        //                 None
        //             }
        //         }
        //     })
        //     .collect::<Vec<_>>();
        // let new_books = create_book_vec(&epub_paths);
        // if new_books.len() != 0 {
        //     // handled lining up the data to 'skip' sorting it
        //     // let mut index_offset = 0;
        //     // for (book, index) in &new_books {
        //     //     current_books.insert(index + index_offset, book.to_owned());
        //     //     index_offset += 1;
        //     // }
        //     let new_books: Vec<&Book> = new_books.iter().map(|book| book).collect();

        //     match insert_book_db_batch(new_books) {
        //         Ok(_) => println!("Insert worked"),
        //         Err(_) => println!("INsert not so work"),
        //     };
        //     file_changes = true;
        //  }
        // } else {
        //     println! {"No new"}
        // }
        // } else {
        //     current_books = create_book_vec(&epub_paths);
        //     let book_refs: Vec<&Book> = current_books.iter().collect();
        //     match insert_book_db_batch(book_refs) {
        //         Ok(_) => println!("Insert worked"),
        //         Err(_) => println!("INsert not so work"),
        //     };
        //     file_changes = true;
        // }

        // if file_changes {
        //     let file =
        //         File::create(json_path).expect("JSON path should be defined, and a valid path");

        //     serde_json::to_writer_pretty(file, &current_books).expect("The book JSON should exist");
        // }
        //
        // println!("Execution time: {} ms", start_time.elapsed().as_millis());

        // Some(current_books)
    }
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

    for line in std::io::BufRead::lines(reader) {
        let line_content = line.unwrap();
        let split: Vec<&str> = line_content.split('=').collect();

        if split.len() == 2 {
            settings_map.insert(split[0].to_string(), split[1].to_string());
        }
    }

    settings_map
}
pub fn create_default_settings() -> Result<(), Error> {
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
