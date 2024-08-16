use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous};

use time::{format_description::parse, OffsetDateTime};
use tokio::sync::OnceCell;

use crate::{
    book_item::{insert_book_db_batch, Book},
    book_worker::{get_cache_dir, get_dump_json_path},
};

static DB: OnceCell<SqlitePool> = OnceCell::const_new();

async fn create_pool() -> SqlitePool {
    let database_name = env!("DATABASE_FILENAME");
    let db_location = get_cache_dir().join(database_name);

    let options = SqliteConnectOptions::new()
        .filename(&db_location) // Use the correct database file path
        .create_if_missing(true)
        .synchronous(SqliteSynchronous::Off)
        .journal_mode(SqliteJournalMode::Memory);

    let pool = SqlitePool::connect_with(options).await.expect(&format!(
        "could not connect to database_url: {:?}",
        db_location
    ));

    sqlx::migrate!(".\\migrations")
        .run(&pool)
        .await
        .expect("migrations failed");
    _ = import_book_json();
    pool
}
fn append_date_to_filename(file_path: &str) -> String {
    // Get the current date in YYYY-MM-DD format
    let format = parse("[year][month][day]").unwrap();

    // Get the current date in YYYYMMDD format
    let today = OffsetDateTime::now_utc().format(&format).unwrap();
    if let Some(pos) = file_path.rfind('.') {
        // Split the file name and extension, and insert the date before the extension
        format!("{}_{today}{}", &file_path[..pos], &file_path[pos..])
    } else {
        // If there's no extension, just append the date to the file path
        format!("{}_{}", file_path, today)
    }
}
pub fn import_book_json() -> Result<(), std::io::Error> {
    if let Some(backup_path) = get_dump_json_path() {
        let path = Path::new(&backup_path);

        if path.exists() {
            let file = File::open(&backup_path)?;
            let old_books: Vec<Book> = match serde_json::from_reader(BufReader::new(file)) {
                Ok(data) => data,
                Err(_) => Vec::new(),
            };

            match insert_book_db_batch(&old_books) {
                Ok(()) => {
                    println!("Restored backup containing {:?} books!", &old_books.len());
                    let spent_file_name = append_date_to_filename(
                        &backup_path
                            .to_str()
                            .expect("how did the backup path vanish"),
                    );
                    fs::rename(&backup_path, spent_file_name)?;
                }
                Err(_) => {
                    println!("Hurray, something went wrong while restoring the backup");
                }
            };
        } else {
            println!("Backup path does not exist");
        }
    } else {
        println!("Backup path not provided");
    }
    Ok(())
}

pub async fn init_db() {
    //TODO: Handle result haha

    DB.set(create_pool().await)
        .expect("Fail to init DB, is the server running in the same node?");
}

pub fn get_db<'a>() -> &'a SqlitePool {
    DB.get().expect("database not ready")
}
