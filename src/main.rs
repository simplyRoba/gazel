use tracing::info;
use tracing_subscriber::EnvFilter;

use gazel::auth::Authentication;
use gazel::config::Config;
use gazel::state::AppState;
use gazel::{db, server};

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Invalid configuration");

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting gazel v{}", env!("CARGO_PKG_VERSION"));
    info!(port = config.port, db_path = %config.db_path, log_level = %config.log_level, "Configuration loaded");

    let auth = match config.auth.clone() {
        Some(auth_config) => Some(
            Authentication::bootstrap(auth_config)
                .await
                .expect("Failed to initialize OIDC authentication"),
        ),
        None => None,
    };

    let pool = db::create_pool(&config.db_path)
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    let state = AppState::new(pool, auth);
    let router = server::router(state);

    server::serve(router, config.port)
        .await
        .expect("Server error");
}
