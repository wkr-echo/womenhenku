pub mod error;
pub mod migration;
pub mod model;
pub mod repository;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::{Path, PathBuf};

/// Type alias for the SQLite connection pool used throughout the application.
pub type DbPool = Pool<SqliteConnectionManager>;

/// Returns the default database path: `{data_local_dir}/mercury/mercury.db`
pub fn default_db_path() -> PathBuf {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("mercury").join("mercury.db")
}

/// Initialize the database: create directory, open connection pool, run migrations.
pub fn initialize_database(db_path: &Path) -> Result<DbPool, anyhow::Error> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)?;

    // Enable WAL mode and foreign keys on a test connection
    {
        let conn = pool.get()?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    }

    tracing::info!("Database opened at: {}", db_path.display());

    // Run migrations
    migration::run_migrations(&pool)?;

    Ok(pool)
}

/// Open an in-memory database pool for testing.
pub fn open_test_db_pool() -> Result<DbPool, anyhow::Error> {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder().max_size(2).build(manager)?;
    {
        let conn = pool.get()?;
        conn.execute_batch("PRAGMA journal_mode=MEMORY;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    }
    migration::run_migrations(&pool)?;
    Ok(pool)
}

/// Open an in-memory single connection for migration-level tests.
pub fn open_test_connection() -> Result<rusqlite::Connection, anyhow::Error> {
    let conn = rusqlite::Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA journal_mode=MEMORY;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    migration::run_migrations_on_conn(&conn)?;
    Ok(conn)
}
