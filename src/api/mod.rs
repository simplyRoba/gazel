pub mod error;
pub mod vehicles;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/vehicles", get(vehicles::list).post(vehicles::create))
        .route(
            "/vehicles/{id}",
            get(vehicles::get)
                .put(vehicles::update)
                .patch(vehicles::patch)
                .delete(vehicles::delete),
        )
}
