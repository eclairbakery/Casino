use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Connection, Pool, Sqlite, SqliteConnection};
use std::error::Error;
use std::fs::File;

pub async fn init(db_name: &str) -> Result<(), Box<dyn Error>> {
    File::create(db_name)?;

    let mut conn = SqliteConnection::connect(db_name).await?;

    let tables = [
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            cash FLOAT NOT NULL DEFAULT 0,
            bank FLOAT NOT NULL DEFAULT 0
        )",
        "CREATE TABLE IF NOT EXISTS timeouts (
            user_id INTEGER PRIMARY KEY,
            last_crime INTEGER NOT NULL DEFAULT 0,
            last_rob INTEGER NOT NULL DEFAULT 0,
            last_slut INTEGER NOT NULL DEFAULT 0,
            last_work INTEGER NOT NULL DEFAULT 0,
            last_hazarded INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
    ];

    let mut tx = conn.begin().await?;

    for table_sql in &tables {
        sqlx::query(table_sql).execute(tx.as_mut()).await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn create_pool(db_name: &str) -> Result<Pool<Sqlite>, Box<dyn Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(format!("sqlite://{db_name}").as_str())
        .await?;

    Ok(pool)
}
