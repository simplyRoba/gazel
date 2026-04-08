use std::time::Instant;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing::{debug, info};

use crate::embedded::static_handler;
use crate::state::AppState;

/// Build the application router.
///
/// - `/health` — database connectivity check (outside `/api`)
/// - `/api/*`  — domain API routes (empty for now)
/// - fallback  — embedded `SvelteKit` SPA
pub fn router(state: AppState) -> Router {
    let pool = state.pool.clone();

    Router::new()
        .route("/health", get(move || health(pool)))
        .nest("/api", crate::api::router(state.clone()))
        .fallback(static_handler)
        .layer(middleware::from_fn(access_log))
        .with_state(state)
}

/// Access-log middleware. Logs every request at `debug` level with method,
/// path, status code, and elapsed time.
async fn access_log(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status();
    let duration = start.elapsed();
    debug!("{method} {path} → {status} ({duration:.1?})");

    response
}

/// Health check handler. Verifies database connectivity by executing
/// `SELECT 1` and returns the application version.
async fn health(pool: SqlitePool) -> Response {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION")
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unhealthy" })),
        )
            .into_response(),
    }
}

/// Start the HTTP server on the given port with graceful shutdown.
///
/// # Errors
///
/// Returns an error if the TCP listener cannot bind to the port.
pub async fn serve(router: Router, port: u16) -> std::io::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

/// Await either SIGINT (Ctrl+C) or SIGTERM, then return so the server can
/// drain in-flight requests.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { info!("Received SIGINT, shutting down"); }
        () = terminate => { info!("Received SIGTERM, shutting down"); }
    }
}
