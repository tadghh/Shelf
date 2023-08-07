use std::fs;
use std::{ fs::OpenOptions, path::Path };
use std::io::{ BufReader, Write };
use serde::{ Deserialize, Serialize };
use epub::doc::EpubDoc;
use regex::Regex;

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
            println!("yo");
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

// fn create_cover(book_directory: String, write_directory: &String) -> String {
//     use rand::Rng;

//     let mut rng = rand::thread_rng();

//     let random_num = rng.gen_range(0..=10000).to_string();
//     let cover_path = format!("{}/{}.jpg", &write_directory, random_num);
//     let doc = EpubDoc::new(book_directory);
//     let mut doc = doc.unwrap();
//     if let Some(cover) = doc.get_cover() {
//         let cover_data = cover.0;
//         let f = fs::File::create(&cover_path);
//         let mut f = f.unwrap();
//         if let Err(err) = f.write_all(&cover_data) {
//             eprintln!("Failed to write cover data: {:?}", err);
//         }
//     }

//      cover_path
// }
fn create_cover(book_directory: String, write_directory: &String) -> Result<String, String> {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    let random_num = rng.gen_range(0..=10000).to_string();
    let cover_path = format!("{}/{}.jpg", &write_directory, random_num);

    let mut doc = EpubDoc::new(&book_directory).map_err(|err|
        format!("Error opening EpubDoc: {}", err)
    )?;
    println!("{:?}", doc.resources);

    if let Some(cover) = doc.get_cover() {
        // println!("COver info {:?} Title: \n", doc);
        let cover_data = cover.0;
        let mut f = fs::File
            ::create(&cover_path)
            .map_err(|err| format!("Error creating cover file: {}", err))?;
        f.write_all(&cover_data).map_err(|err| format!("Error writing cover data: {}", err))?;
    } else {
        //Sometimes the cover isnt setup correctly lets manually look for it
        let mut doc2 = EpubDoc::new(&book_directory).map_err(|err|
            format!("Error opening EpubDoc: {}", err)
        )?;
        let test2 = &mut doc.resources;
        let pattern = r"cover";
        let regex = Regex::new(pattern).unwrap();
        // let test = test2.resources;
        let cover_id = test2.keys().find(|key| regex.is_match(key));

        //regex map resources for the path
        let cover = doc2.get_resource(cover_id.unwrap());
        let cover_data = cover.unwrap().0;
        let mut f = fs::File
            ::create(&cover_path)
            .map_err(|err| format!("Error creating cover file: {}", err))?;
        f.write_all(&cover_data).map_err(|err| format!("Error writing cover data: {}", err))?;
        println!("{:?}", doc.resources);
        // println!("{:?}", doc.get_resource("id_cover_jpg"));

        //cover_path = format!("{}/{}", get_home_dir(), "error.jpg");

        //return Err("No cover image found in the EpubDoc".to_string());
    }

    Ok(cover_path)
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
