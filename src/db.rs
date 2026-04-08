use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

/// Create a `SQLite` connection pool configured for a single-user workload.
///
/// - WAL journal mode for concurrent reads during writes
/// - 5-second busy timeout to tolerate brief contention
/// - Up to 5 connections
/// - Auto-creates the database file and parent directories if missing
///
/// # Errors
///
/// Returns an error if the pool cannot be established (e.g. invalid path,
/// permission issues).
pub async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create parent directories for the database file (skip for in-memory).
    if db_path != ":memory:"
        && let Some(parent) = std::path::Path::new(db_path).parent()
    {
        std::fs::create_dir_all(parent).ok();
    }

    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// Run all pending `SQLx` migrations against the pool.
///
/// # Errors
///
/// Returns an error if any migration fails to apply.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
