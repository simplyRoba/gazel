use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new("info").unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Starting gazel v{}", env!("CARGO_PKG_VERSION"));
}
