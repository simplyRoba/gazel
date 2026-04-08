use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

/// Static assets from the `SvelteKit` build, embedded into the binary.
#[derive(Embed)]
#[folder = "ui/build/"]
struct Assets;

/// Serve embedded static files with SPA fallback.
///
/// 1. Try an exact path match against the embedded assets.
/// 2. If no match, serve `index.html` so `SvelteKit` handles client-side routing.
/// 3. If `index.html` is also missing (should not happen), return 404.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Exact asset match.
    if let Some(file) = Assets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime)],
            file.data.to_vec(),
        )
            .into_response();
    }

    // SPA fallback — serve index.html for client-side routing.
    if let Some(file) = Assets::get("index.html") {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html".to_string())],
            file.data.to_vec(),
        )
            .into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}
