use base64::{ engine::general_purpose, Engine as _ };
use epub::doc::EpubDoc;
use regex::Regex;

use crate::shelf::get_cover_image_folder_name;

use super::Book;
use std::{ env, fs::File, io::{ Read, BufReader }, cmp::Ordering, collections::HashMap };

/// Returns Shelfs current working directory
pub fn get_home_dir() -> String {
    match env::current_dir() {
        Ok(dir) => dir.to_string_lossy().replace('\\', "/"),
        Err(_) => String::new(), // Return an empty string as a default value
    }
}

/// Removes special charectars from a given string and returns it
/// Some book titles contain charectars that arent compatible when used as filenames
///
/// # Arguments
///
/// * `filename` - The filename to sanitize
///
pub fn sanitize_windows_filename(filename: String) -> String {
    let disallowed_chars: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    let sanitized: String = filename
        .chars()
        .map(|c| if disallowed_chars.contains(&c) { '_' } else { c })
        .collect();

    sanitized
}

/// Returns the configured cover directory
pub fn get_covers_directory() -> String {
    format!("{}/{}/{}", get_home_dir(), "cache", get_cover_image_folder_name())
}

//This isnt needed
pub fn base64_encode_book(file_path: &str) -> Result<String, String> {
    let mut buffer = Vec::new();

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            match
                File::open(
                    env
                        ::current_exe()
                        .expect("Failed to get current executable path")
                        .parent()
                        .expect("Failed to get parent directory")
                )
            {
                Ok(file) => file,
                Err(err) => {
                    panic!("Failed to error: {}", err);
                }
            }
        }
    };

    match file.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(err) => {
            return Err(format!("Failed to read file: {}", err));
        }
    }

    // Encode the file data as base64
    let base64_data = general_purpose::STANDARD.encode(&buffer);
    Ok(base64_data)
}

/// Encodes the data of a give file, returning the encoded data
/// This is to get around CORS issues
///
/// # Arguments
///
/// * `filepath` - The file to encode
///
#[tauri::command(rename_all = "snake_case")]
pub fn base64_encode_file(file_path: &str) -> Result<String, String> {
    let mut buffer = Vec::new();

    //Refactor this
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            match
                File::open(
                    env
                        ::current_exe()
                        .expect("Failed to get current executable path")
                        .parent()
                        .expect("Failed to get parent directory")
                        .join("error.jpg")
                )
            {
                Ok(file) => file,
                Err(err) => {
                    panic!("Failed to open error.jpg: {}", err);
                }
            }
        }
    };

    file.read_to_end(&mut buffer).expect("There was an issue with the buffer");

    // Encode the file data as base64
    let base64_data = general_purpose::STANDARD_NO_PAD.encode(&buffer);
    Ok(base64_data)
}

// #[tauri::command(rename_all = "snake_case")]
// pub fn base64_encode_covers() -> Result<Vec<String>, String> {
//     let base64_image_addresses: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
//     let handles: Vec<_> = fs
//         ::read_dir(get_covers_directory())
//         .map_err(|err| format!("Failed to read directory: {}", err))?
//         .map(|entry| {
//             let base64_image_addresses = Arc::clone(&base64_image_addresses);
//             thread::spawn(move || {
//                 let entry = entry.map_err(|err| format!("Error reading directory entry: {}", err))?;
//                 if let Some(file_name) = entry.file_name().to_str() {
//                     if file_name.ends_with(".jpg") || file_name.ends_with(".png") {
//                         let mut buffer = Vec::new();
//                         let file_path = entry.path();
//                         let mut file = std::fs::File
//                             ::open(&file_path)
//                             .map_err(|err|
//                                 format!("Failed to open {}: {}", file_path.display(), err)
//                             )?;
//                         file
//                             .read_to_end(&mut buffer)
//                             .map_err(|err|
//                                 format!("Failed to read {}: {}", file_path.display(), err)
//                             )?;
//                         let base64_data = general_purpose::STANDARD_NO_PAD.encode(&buffer);
//                         let mut addresses = base64_image_addresses.lock().unwrap();
//                         addresses.push(base64_data);
//                     }
//                 }
//                 Ok(()) as Result<(), String>
//             })
//         })
//         .collect();

//     for handle in handles {
//         handle.join().unwrap()?;
//     }

//     let addresses = Arc::try_unwrap(base64_image_addresses)
//         .map_err(|_| "Mutex still locked".to_string())?
//         .into_inner()
//         .map_err(|_| "Mutex still locked".to_string())?;

//     Ok(addresses)
// }

/// Finds a chunk in the dataset that starts with the same letter as the key, returning the found value
/// One this chunk is found we binary search within that section, theoretically faster
///
/// # Arguments
///
/// * `dataset` - A dataset sorted in alphabetical order
/// * `key` - The key to look for
///
pub fn chunk_binary_search_index(dataset: &Vec<Book>, key: &String) -> Option<usize> {
    let title = key.to_string();
    //handel lower case
    let low = dataset.iter().position(|b| b.title[..1] == title[..1]);

    if let Some(index) = low {
        let mut high = dataset
            .iter()
            .rposition(|b| b.title[..1] == title[..1])
            .unwrap();
        let mut unwrapped_low = index;
        while unwrapped_low <= high {
            let mid = (unwrapped_low + high) / 2;
            match dataset[mid].title.cmp(&title) {
                Ordering::Equal => {
                    return None;
                }
                Ordering::Less => {
                    unwrapped_low = mid + 1;
                }
                Ordering::Greater => {
                    high = mid - 1;
                }
            }
        }
        Some(unwrapped_low)
    } else {
        Some(dataset.len())
    }
}

/// Finds a chunk in the dataset that starts with the same letter as the key, returning the found index
/// One this chunk is found we binary search within that section, theoretically faster
///
/// # Arguments
///
/// * `dataset` - A dataset sorted in alphabetical order
/// * `key` - The key to look for
///
pub fn chunk_binary_search_index_load(dataset: &[Book], key: &String) -> Option<usize> {
    let title = key.to_string();
    //handel lower case
    let low = dataset.iter().position(|b| b.title[..1] == title[..1]);

    if let Some(index) = low {
        let mut high = dataset
            .iter()
            .rposition(|b| b.title[..1] == title[..1])
            .unwrap();
        let mut unwrapped_low = index;
        while unwrapped_low <= high {
            let mid = (unwrapped_low + high) / 2;
            match dataset[mid].title.cmp(&title) {
                Ordering::Equal => {
                    return Some(mid);
                }
                Ordering::Less => {
                    unwrapped_low = mid + 1;
                }
                Ordering::Greater => {
                    high = mid - 1;
                }
            }
        }
        Some(unwrapped_low)
    } else {
        None
    }
}

/// Looks in an Epubs resources for a given key with a given mime type, returning the data as a string
///
/// # Arguments
///
/// * `key_regex` - The key to look for as regex query
/// * `mime_regex` - The mime type the key should have as a regex query
/// * `epub_resources` - The epubs resources to look in
/// * `doc` - A EpubDoc object to help with getting the resources
///
pub fn check_epub_resource(
    key_regex: Regex,
    mime_regex: Regex,
    epub_resources: &HashMap<String, (std::path::PathBuf, String)>,
    doc: &mut EpubDoc<BufReader<File>>
) -> Option<String> {
    epub_resources
        .keys()
        .find(
            |key| key_regex.is_match(key) && mime_regex.is_match(&doc.get_resource(key).unwrap().1)
        )
        .map(|key| key.to_owned())
}
