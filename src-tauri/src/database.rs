use crate::config::Config;
use crate::system;
use anyhow::Result;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::sync::RwLock;

/// A lock containing the pool of SQLite database connections.
static POOL: RwLock<Option<Pool<Sqlite>>> = RwLock::new(None);

/// Initializes the SQLite database pool.
pub async fn initialize(config: &Config) -> Result<()> {
    // Resolve the database file path.
    let db_file = system::resolve_path(&config.data_file)?;

    // If the file doesn't exist, create the database file.
    if !db_file.exists() {
        let _ = File::create(&db_file)?;
    }

    // Create the database connection pool.
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_file.to_string_lossy())).await?;

    // Run migrations to initialize the database.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Write the pool to the lock.
    *POOL.write().unwrap() = Some(pool);

    Ok(())
}

/// Retrieves a reference to the SQLite database pool.
pub fn get_pool() -> Pool<Sqlite> {
    let pool = POOL.read().unwrap();
    pool.clone().expect("database pool should be initialized")
}
