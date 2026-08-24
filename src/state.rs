use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::auth::Authentication;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// `SQLite` connection pool.
    pub pool: SqlitePool,
    /// Process-local OIDC/session state when authentication is enabled.
    pub auth: Option<Arc<Authentication>>,
}

impl AppState {
    /// Construct application state for either authentication mode.
    pub fn new(pool: SqlitePool, auth: Option<Authentication>) -> Self {
        Self {
            pool,
            auth: auth.map(Arc::new),
        }
    }
}

/// Allows handlers to extract `State<SqlitePool>` directly instead of the
/// full `AppState` when only the pool is needed.
impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
