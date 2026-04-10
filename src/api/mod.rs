pub mod error;
pub mod export;
pub mod fillups;
pub mod import;
pub mod settings;
pub mod stats;
pub mod vehicles;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

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
        .route("/vehicles/{id}/export", get(export::export_vehicle))
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
        .route("/vehicles/{vehicle_id}/stats", get(stats::summary))
        .route("/vehicles/{vehicle_id}/stats/history", get(stats::history))
        .route("/export", get(export::export_all))
        // Import endpoint with 10 MB body size limit (returns 413 if exceeded)
        .route(
            "/import",
            post(import::import_data).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
}
