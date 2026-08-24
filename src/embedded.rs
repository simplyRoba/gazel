use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

/// Static assets from the `SvelteKit` build, embedded into the binary.
#[derive(Embed)]
#[folder = "ui/build/"]
struct Assets;

const PUBLIC_ROOT_ASSETS: &[&str] = &[
    "favicon.svg",
    "manifest.json",
    "icon-180.png",
    "icon-192.png",
    "icon-192-dark.png",
    "icon-512.png",
    "icon-512-dark.png",
    "icon-1024.png",
    "icon-1024-dark.png",
];

/// Return exact non-HTML asset route paths required by the public login page.
pub fn public_asset_paths() -> Vec<String> {
    Assets::iter()
        .filter_map(|path| {
            let path = path.as_ref();
            is_public_asset(path).then(|| format!("/{path}"))
        })
        .collect()
}

/// Serve one exact non-HTML embedded asset without SPA fallback.
pub async fn exact_public_asset_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if !is_public_asset(path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    exact_asset(path).unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

/// Serve the embedded SPA document.
pub async fn index_handler() -> Response {
    exact_asset("index.html").unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

/// Serve embedded static files with SPA fallback.
///
/// Disabled authentication and authenticated application routes retain the
/// existing behavior: exact assets are preferred and every miss receives the
/// SPA document for client-side routing.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    exact_asset(path).unwrap_or_else(|| {
        exact_asset("index.html").unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
    })
}

fn exact_asset(path: &str) -> Option<Response> {
    let file = Assets::get(path)?;
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    Some(
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime)],
            file.data.to_vec(),
        )
            .into_response(),
    )
}

fn is_public_asset(path: &str) -> bool {
    !is_html(path) && (path.starts_with("_app/") || PUBLIC_ROOT_ASSETS.contains(&path))
}

fn is_html(path: &str) -> bool {
    mime_guess::from_path(path)
        .first_raw()
        .is_some_and(|mime| mime == "text/html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_asset_allowlist_is_limited_to_login_resources() {
        for path in [
            "_app/immutable/entry/start.js",
            "_app/immutable/assets/app.css",
            "favicon.svg",
            "manifest.json",
            "icon-180.png",
        ] {
            assert!(is_public_asset(path));
        }

        for path in [
            "index.html",
            "_app/fallback.html",
            "robots.txt",
            "export.json",
            "future-resource.txt",
        ] {
            assert!(!is_public_asset(path));
        }
    }
}
