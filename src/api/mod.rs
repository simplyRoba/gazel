pub mod error;

use axum::Router;

use crate::state::AppState;

/// Build the `/api` sub-router.
///
/// Domain-specific routes (vehicles, fill-ups, etc.) will be added here
/// in later changes.
pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
}
