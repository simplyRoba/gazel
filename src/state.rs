use axum::extract::FromRef;
use sqlx::SqlitePool;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// `SQLite` connection pool.
    pub pool: SqlitePool,
}

/// Allows handlers to extract `State<SqlitePool>` directly instead of the
/// full `AppState` when only the pool is needed.
impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
