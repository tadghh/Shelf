use std::fs::{ self, File };
use std::{ fs::OpenOptions, path::Path };
use std::io::{ BufReader, Write };
use serde::{ Deserialize, Serialize };
use epub::doc::EpubDoc;
use regex::Regex;
use xmltree::Element;

use crate::book::bookio::get_home_dir;
use crate::book::util::{ chunk_binary_search_index_load, base64_encode_book, base64_encode_file };
//use crate::shelf::get_configuration_option;
pub mod bookio;
pub mod util;
struct BookCache {
    books: Vec<Book>,
    json_path: String,
}

impl BookCache {
    fn update_path(&mut self, new_json_path: String) {
        self.json_path = new_json_path;
    }
    fn update_books(&mut self, new_books: Vec<Book>) {
        self.books = new_books;
    }
}
static mut BOOK_JSON: BookCache = BookCache {
    books: Vec::new(),
    json_path: String::new(),
};
#[derive(Serialize, Deserialize, Debug)]
pub struct Book {
    cover_location: String,
    book_location: String,
    title: String,
}

#[tauri::command]
pub fn load_book(title: String) -> Result<String, String> {
    unsafe {
        let open_file: &String = &BOOK_JSON.json_path.to_owned();
        println!("{:?}", Path::new(&open_file).exists());
        println!("{:?}", &BOOK_JSON.json_path);
        if Path::new(&open_file).exists() {
            let file = OpenOptions::new().read(true).write(true).create(true).open(open_file);

            BOOK_JSON.update_books(match serde_json::from_reader(BufReader::new(file.unwrap())) {
                Ok(data) => data,
                Err(_) => Vec::new(),
            });
            //  println!("Yo Index {:?}", &BOOK_JSON.books.take());
            //Okay we have it but like dont steal the data perhaps?
            let temp = &BOOK_JSON.books;
            let book_index = chunk_binary_search_index_load(temp, &title);
            if let Some(book) = temp.get(book_index.unwrap()) {
                // Accessing the book at the specified index
                println!("{}", book.book_location);
                return Ok(base64_encode_book(&book.book_location.to_string()).unwrap());
            } else {
                println!("Invalid index");
            }
        } else {
            return Err("JSON File missing".to_string());
        }
    }

    Err("Error occured".to_string())
}

fn sanitize_windows_filename(filename: String) -> String {
    let disallowed_chars: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    let sanitized: String = filename
        .chars()
        .map(|c| if disallowed_chars.contains(&c) { '_' } else { c })
        .collect();

    sanitized
}
fn find_cover(
    root: &Element,
    mut doc: EpubDoc<BufReader<File>>,
    mime_type_regex: &Regex,
    cover_path: &str
) -> Result<(), String> {
    let epub_resources = doc.resources.clone();
    match find_img_element(root) {
        Some(img_element) => {
            if let Some(image_src) = img_element.attributes.get("src") {
                if
                    let (Some(last_slash), Some(last_dot)) = (
                        image_src.rfind('/'),
                        image_src.rfind('.'),
                    )
                {
                    let filename = &image_src[last_slash + 1..last_dot];
                    let cover_key_regex = Regex::new(filename).unwrap();

                    if
                        let Some(cover_id) = epub_resources
                            .keys()
                            .find(
                                |key|
                                    cover_key_regex.is_match(key) &&
                                    mime_type_regex.is_match(&doc.get_resource(key).unwrap().1)
                            )
                    {
                        if let Some(cover) = doc.get_resource(cover_id) {
                            let cover_data = cover.0;
                            let mut f = fs::File
                                ::create(cover_path)
                                .map_err(|err| format!("Error creating cover file: {}", err))?;
                            f
                                .write_all(&cover_data)
                                .map_err(|err| format!("Error writing cover data: {}", err))?;
                        } else {
                            return Err("Failed to get cover resource".to_string());
                        }
                    } else {
                        return Err("No cover ID found".to_string());
                    }
                } else {
                    return Err("Slash or dot not found in the string".to_string());
                }
            } else {
                return Err("Image element has no 'src' attribute".to_string());
            }
        }
        None => {
            return Err("Image element not found".to_string());
        }
    }

    Ok(())
}
fn create_cover(book_directory: String, write_directory: &String) -> Result<String, String> {
    let mut doc = EpubDoc::new(book_directory).map_err(|err|
        format!("Error opening EpubDoc: {}", err)
    )?;

    //Base filename off the books title
    let cover_path = format!(
        "{}/{}.jpg",
        &write_directory,
        sanitize_windows_filename(doc.mdata("title").unwrap())
    );

    if doc.get_cover().is_some() {
        let cover_data = doc.get_cover().unwrap();
        let mut f = fs::File
            ::create(&cover_path)
            .map_err(|err| format!("Error creating cover file: {}", err))?;

        f.write_all(&cover_data.0).map_err(|err| format!("Error writing cover data: {}", err))?;
    } else {
        //Look for the cover_id in the epub, we are just looking for any property containing the word cover
        //This is because EpubDoc looks for an exact string, and some epubs dont contain it
        // let mimetype = r"image/jpeg";
        println!("We processing fr {}", cover_path);

        let cover_key_regex = Regex::new(r"(?i)cover").unwrap();
        let mime_type_regex = Regex::new(r"image/jpeg").unwrap();

        let epub_resources = doc.resources.clone();
        let cover_id = epub_resources
            .keys()
            .find(
                |key|
                    cover_key_regex.is_match(key) &&
                    mime_type_regex.is_match(&doc.get_resource(key).unwrap().1)
            );

        if let Some(..) = cover_id {
            let cover = doc.get_resource(cover_id.unwrap());
            let cover_data = cover.unwrap().0;
            let mut f = fs::File
                ::create(&cover_path)
                .map_err(|err| format!("Error creating cover file: {}", err))?;
            f.write_all(&cover_data).map_err(|err| format!("Error writing cover data: {}", err))?;
        } else {
            //let xhtml_mime_regex = Regex::new(r"application/xhtml+xml").unwrap();

            //We should make sure its xhtml
            let cover_id = epub_resources.keys().find(|key| cover_key_regex.is_match(key));
            let resource = doc.get_resource(cover_id.unwrap());
            let file_content = &resource.unwrap().0;
            let buffer_str = String::from_utf8_lossy(file_content);

            // Parse the XML content
            let root = Element::parse(buffer_str.as_bytes()).expect("Failed to parse XML");
            if find_cover(&root, doc, &mime_type_regex, &cover_path).is_err() {
                //No cover was found
                return Ok(format!("{}/{}", get_home_dir(), "error.jpg"));
            }
        }
    }

    Ok(cover_path)
}
fn find_img_element(element: &Element) -> Option<&Element> {
    if element.name == "img" {
        Some(element)
    } else {
        for child in &element.children {
            if let Some(child_element) = child.as_element() {
                if let Some(img_element) = find_img_element(child_element) {
                    return Some(img_element);
                }
            }
        }
        None
    }
}
#[tauri::command(rename_all = "snake_case")]
pub fn get_cover(book_title: String) -> Result<String, String> {
    unsafe {
        let open_file: &String = &BOOK_JSON.json_path.to_owned();

        if Path::new(&open_file).exists() {
            let file = OpenOptions::new().read(true).write(true).create(true).open(open_file);

            BOOK_JSON.update_books(match serde_json::from_reader(BufReader::new(file.unwrap())) {
                Ok(data) => data,
                Err(_) => Vec::new(),
            });

            let temp = &BOOK_JSON.books;
            let book_index = chunk_binary_search_index_load(temp, &book_title);
            println!("yo");
            if let Some(book) = temp.get(book_index.unwrap()) {
                return Ok(base64_encode_file(&book.cover_location.to_string()).unwrap());
            } else {
                println!("Invalid index");
            }
        } else {
            return Err("JSON File missing".to_string());
        }
    }
    Err("Error occured".to_string())
}
