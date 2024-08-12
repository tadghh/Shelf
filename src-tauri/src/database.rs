use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use tokio::sync::OnceCell;

use crate::book_worker::get_cache_dir;

static DB: OnceCell<SqlitePool> = OnceCell::const_new();

async fn create_pool() -> SqlitePool {
    let database_url = env!("DATABASE_URL");
    let db_location = get_cache_dir().join("book.db");
    let options = SqliteConnectOptions::new()
        .filename(db_location) // Use the correct database file path
        .create_if_missing(true)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Off)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Memory);

    let pool = SqlitePool::connect_with(options).await.expect(&format!(
        "could not connect to database_url: {}",
        database_url
    ));
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");
    pool
}

pub async fn init_db() {
    //TODO: Handle result haha

    DB.set(create_pool().await)
        .expect("Fail to init DB, is the server running in the same node?");
}

pub fn get_db<'a>() -> &'a SqlitePool {
    DB.get().expect("database not ready")
}
