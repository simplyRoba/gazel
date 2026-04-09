pub mod error;
pub mod fillups;
pub mod settings;
pub mod vehicles;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/settings",
            get(settings::get_settings).put(settings::update_settings),
        )
        .route("/vehicles", get(vehicles::list).post(vehicles::create))
        .route(
            "/vehicles/{id}",
            get(vehicles::get)
                .put(vehicles::update)
                .patch(vehicles::patch)
                .delete(vehicles::delete),
        )
        .route(
            "/vehicles/{vehicle_id}/fillups",
            get(fillups::list).post(fillups::create),
        )
        .route(
            "/vehicles/{vehicle_id}/fillups/{id}",
            get(fillups::get)
                .put(fillups::update)
                .delete(fillups::delete),
        )
}
