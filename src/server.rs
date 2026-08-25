use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use sqlx::SqlitePool;
use time::{Duration, OffsetDateTime};
use tokio::net::TcpListener;
use tower_sessions::Session;
use tower_sessions::cookie::{Cookie, SameSite};
use tracing::{debug, info};

use crate::api::error::ApiError;
use crate::auth::Authentication;
use crate::embedded::{
    exact_public_asset_handler, index_handler, public_asset_paths, static_handler,
};
use crate::state::AppState;

/// Build the application router.
///
/// Disabled mode preserves the existing API and SPA fallback. Enabled mode
/// composes exact public login resources separately from one protected API and
/// application router, all under a single private session layer.
pub fn router(state: AppState) -> Router {
    let router = match state.auth.clone() {
        Some(authentication) => enabled_router(&state, &authentication),
        None => disabled_router(state),
    };
    router.layer(middleware::from_fn(access_log))
}

fn disabled_router(state: AppState) -> Router {
    let pool = state.pool.clone();
    Router::new()
        .route("/health", get(move || health(pool)))
        .route("/auth/config", get(|| async { disabled_auth_config() }))
        .route("/api/info", get(|| async { info(false) }))
        .nest("/api", crate::api::router(state.clone()))
        .fallback(static_handler)
        .with_state(state)
}

fn enabled_router(state: &AppState, authentication: &Arc<Authentication>) -> Router {
    let protected = Router::new()
        .route("/api/info", get(|| async { info(true) }))
        .nest("/api", crate::api::router(state.clone()))
        .fallback(static_handler)
        .layer(middleware::from_fn_with_state(
            Arc::clone(authentication),
            require_authentication,
        ))
        .with_state(state.clone());

    let pool = state.pool.clone();
    let provider_name = authentication.config().provider_name.clone();
    let secure_cookie = authentication.config().secure_cookie();
    let login_authentication = Arc::clone(authentication);
    let mut public = Router::new()
        .route("/health", get(move || health(pool)))
        .route(
            "/auth/config",
            get(move || {
                let provider_name = provider_name.clone();
                async move { enabled_auth_config(&provider_name) }
            }),
        )
        .route(
            "/login",
            get(move |session| login_page(Arc::clone(&login_authentication), session)),
        )
        .route(
            "/auth/logout",
            post(move |session| logout(session, secure_cookie)),
        )
        .merge(authentication.protocol_routes());

    for path in public_asset_paths() {
        public = public.route(&path, get(exact_public_asset_handler));
    }

    public
        .merge(protected)
        .layer(authentication.session_layer())
}

async fn require_authentication(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let cookie_header_present = request.headers().contains_key(header::COOKIE);
    let gazel_session_cookie_present =
        request_has_named_cookie(request.headers(), crate::auth::SESSION_COOKIE_NAME);
    let authentication_status = authentication.authentication_status(&session).await;
    let authenticated = authentication_status.is_authenticated();

    debug!(
        request_path = path,
        cookie_header_present,
        gazel_session_cookie_present,
        tower_session_id_resolved = authentication_status.tower_session_id_resolved,
        tower_session_record = authentication_status.tower_session_record.as_str(),
        authenticated_record = authentication_status.authenticated_record.as_str(),
        authenticated,
        "Authentication request diagnostics"
    );

    if authenticated {
        return next.run(request).await;
    }

    if path == "/api" || path.starts_with("/api/") {
        return ApiError::Unauthorized("AUTHENTICATION_REQUIRED").into_response();
    }

    let request_target = request
        .uri()
        .path_and_query()
        .map_or("/", axum::http::uri::PathAndQuery::as_str);
    let return_to = crate::auth::validate_return_to(Some(request_target));
    crate::auth::login_redirect(&return_to)
}

async fn login_page(authentication: Arc<Authentication>, session: Session) -> Response {
    if authentication
        .authentication_status(&session)
        .await
        .is_authenticated()
    {
        crate::auth::redirect("/")
    } else {
        index_handler().await
    }
}

async fn logout(session: Session, secure_cookie: bool) -> Response {
    let mut response = match session.flush().await {
        Ok(()) => crate::auth::redirect("/login?logged_out=1"),
        Err(error) => {
            tracing::error!(%error, "Failed to flush authentication session during logout");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    };
    let removal = Cookie::build((crate::auth::SESSION_COOKIE_NAME, ""))
        .http_only(true)
        .secure(secure_cookie)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::ZERO)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build()
        .to_string();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&removal).expect("static session removal cookie should be valid"),
    );
    response
}

fn disabled_auth_config() -> Json<serde_json::Value> {
    Json(json!({ "enabled": false }))
}

fn enabled_auth_config(provider_name: &str) -> Json<serde_json::Value> {
    Json(json!({
        "enabled": true,
        "provider_name": provider_name,
    }))
}

/// Access-log middleware. Logs every request at `debug` level with method,
/// path, status code, and elapsed time.
async fn access_log(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    if response
        .extensions()
        .get::<crate::auth::AuthenticatedCallbackResponse>()
        .is_some()
    {
        debug!(
            request_path = path,
            authenticated_session_established = true,
            gazel_session_set_cookie_present = response_has_named_set_cookie(
                response.headers(),
                crate::auth::SESSION_COOKIE_NAME,
            ),
            "Authentication callback diagnostics"
        );
    }

    let status = response.status();
    let duration = start.elapsed();
    debug!("{method} {path} → {status} ({duration:.1?})");

    response
}

fn request_has_named_cookie(headers: &HeaderMap, name: &str) -> bool {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .any(|pair| {
            pair.split_once('=')
                .is_some_and(|(cookie_name, _)| cookie_name.trim() == name)
        })
}

fn response_has_named_set_cookie(headers: &HeaderMap, name: &str) -> bool {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|value| {
            value
                .split(';')
                .next()
                .and_then(|pair| pair.split_once('='))
                .is_some_and(|(cookie_name, _)| cookie_name.trim() == name)
        })
}

/// Application info handler. Returns version, repository, and license
/// embedded at compile time from `Cargo.toml`.
fn info(auth_enabled: bool) -> Json<serde_json::Value> {
    let mut value = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "license": env!("CARGO_PKG_LICENSE"),
    });
    if auth_enabled {
        value["auth_enabled"] = json!(true);
    }
    Json(value)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_cookie_diagnostic_matches_the_exact_cookie_name() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::COOKIE,
            HeaderValue::from_static("other=1; gazel_session_extra=2"),
        );
        assert!(!request_has_named_cookie(&headers, "gazel_session"));

        headers.append(
            header::COOKIE,
            HeaderValue::from_static("gazel_session=opaque"),
        );
        assert!(request_has_named_cookie(&headers, "gazel_session"));
    }

    #[test]
    fn response_cookie_diagnostic_matches_only_the_set_cookie_name() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_static("other=opaque; gazel_session=attribute-like"),
        );
        assert!(!response_has_named_set_cookie(&headers, "gazel_session"));

        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_static("gazel_session=opaque; HttpOnly; Path=/"),
        );
        assert!(response_has_named_set_cookie(&headers, "gazel_session"));
    }
}
