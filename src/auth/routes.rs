use std::sync::Arc;

use axum::Router;
use axum::extract::{Extension, RawQuery, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use openidconnect::CsrfToken;
use tower_sessions::{Expiry, Session};

use super::Authentication;
use super::return_to::{SafeReturnTo, decode_query_component, return_to_from_query};
use super::runtime::ProtocolError;
use super::session::{AUTHENTICATED_SESSION_KEY, AUTHENTICATED_TTL, AuthenticatedSession};
use super::transaction::{LoginTransaction, TRANSACTION_TTL};

const TRANSACTION_SESSION_KEY: &str = "oidc_transaction";

pub(crate) fn routes(authentication: &Arc<Authentication>) -> Router {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", post(logout))
        .with_state(Arc::clone(authentication))
}

async fn login(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    Extension(super::session::OriginalSessionId(original_id)): Extension<
        super::session::OriginalSessionId,
    >,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let return_to = return_to_from_query(raw_query.as_deref());
    let store = &authentication.sessions.store;
    let now = authentication.clock.now();
    // Loading can clear an unknown ID. Remember known aliases first, and retain
    // the ORIGINAL request revision: reloading must not promote a stale request.
    let known = original_id.and_then(|id| store.for_id(id));
    let Ok(snapshot) = session
        .get::<super::session::Owner>(super::session::OWNER_KEY)
        .await
    else {
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    };
    let (key, lineage) = if let Some(marker) = &snapshot {
        let Some(lineage) = store.lineage(&marker.lineage) else {
            return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
        };
        (marker.lineage.clone(), lineage)
    } else if known.is_some() {
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    } else {
        store.allocate(now + TRANSACTION_TTL)
    };
    let snapshot_id = session.id();
    let _gate = lineage.gate.lock().await;
    if let Some(snapshot) = snapshot {
        let owned = match snapshot_id {
            Some(id) => store.owns(id, &snapshot).await,
            None => false,
        };
        if !owned || store.refresh(&session).await.is_err() || !store.current(&session).await {
            return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
        }
    }
    if authentication.is_authenticated(&session).await {
        return redirect(&return_to);
    }

    let old_state = session
        .get::<String>(TRANSACTION_SESSION_KEY)
        .await
        .ok()
        .flatten();
    let (authorization_url, state, nonce, pkce_verifier) =
        authentication.runtime.authorization_url().await;
    let state_secret = state.secret().clone();
    let now = authentication.clock.now();
    let expires_at = now + TRANSACTION_TTL;
    if authentication
        .transactions
        .replace(
            None,
            state_secret.clone(),
            LoginTransaction {
                nonce,
                pkce_verifier,
                return_to: SafeReturnTo::new(Some(&return_to)),
                expires_at,
            },
            now,
        )
        .await
        .is_err()
    {
        if lineage.authority.lock().await.committed.is_none() {
            store.retire_empty(&key);
        }
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    }

    let marker = super::session::Owner::fresh(key.clone());
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(marker.clone());
        authority.creating = session.id().is_none();
    }
    session.set_expiry(Some(Expiry::AtDateTime(expires_at)));
    let saved = async {
        session.insert(super::session::OWNER_KEY, marker).await?;
        session
            .insert(TRANSACTION_SESSION_KEY, &state_secret)
            .await?;
        session.save().await
    }
    .await;
    if saved.is_err() || !store.current(&session).await {
        authentication
            .transactions
            .consume(&state_secret, now)
            .await;
        store.rollback_binding(&session, &lineage, &key).await;
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    }
    if let Some(old) = old_state {
        authentication.transactions.consume(&old, now).await;
    }
    redirect(authorization_url.as_str())
}

async fn callback(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    Extension(capture): Extension<super::session::ResponseCapture>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let response = complete_callback(&authentication, &session, raw_query.as_deref()).await;
    if let Some(intent) = response
        .extensions()
        .get::<super::session::CallbackIntent>()
    {
        capture.remember_callback(intent.clone());
    }
    response
}

async fn complete_callback(
    authentication: &Authentication,
    session: &Session,
    raw_query: Option<&str>,
) -> Response {
    let Some(query) = CallbackQuery::parse(raw_query) else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    let (returned_state, code) = match query {
        CallbackQuery::Success { state, code } => (state, Some(code)),
        CallbackQuery::ProviderError { state } => (state, None),
    };
    let store = &authentication.sessions.store;
    let Some(lineage) = session.id().and_then(|id| store.for_id(id)) else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    let gate = lineage.gate.lock().await;
    if store.refresh(session).await.is_err() {
        return callback_failure(FailureCode::ProviderUnavailable, "/");
    }
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    }
    let bound_state = session
        .get::<String>(TRANSACTION_SESSION_KEY)
        .await
        .ok()
        .flatten();
    let state_matches = bound_state.is_some_and(|bound_state| {
        CsrfToken::new(bound_state) == CsrfToken::new(returned_state.clone())
    });
    if !state_matches {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    }

    let Some(transaction) = authentication
        .transactions
        .consume(&returned_state, authentication.clock.now())
        .await
    else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    // Consumption only grants exchange permission. The session's committed
    // revision remains the completion guard until replacement/logout/expiry.
    drop(gate);

    let Some(code) = code else {
        return callback_failure(
            FailureCode::AuthenticationFailed,
            transaction.return_to.as_str(),
        );
    };

    match authentication
        .runtime
        .exchange_and_verify(code, &transaction.nonce, transaction.pkce_verifier)
        .await
    {
        Ok(()) => {}
        Err(ProtocolError::Authentication) => {
            return callback_failure(
                FailureCode::AuthenticationFailed,
                transaction.return_to.as_str(),
            );
        }
        Err(ProtocolError::ProviderUnavailable) => {
            return callback_failure(
                FailureCode::ProviderUnavailable,
                transaction.return_to.as_str(),
            );
        }
    }

    persist_callback(
        authentication,
        session,
        &lineage,
        transaction.return_to.as_str(),
    )
    .await
}

async fn persist_callback(
    authentication: &Authentication,
    session: &Session,
    lineage: &super::session::Lineage,
    return_to: &str,
) -> Response {
    let store = &authentication.sessions.store;
    let _gate = lineage.gate.lock().await;
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, return_to);
    }
    if store.refresh(session).await.is_err() {
        let _ = store.revoke(lineage).await;
        session.clear().await;
        let _ = session.flush().await;
        return callback_failure(FailureCode::ProviderUnavailable, return_to);
    }
    let now = authentication.clock.now();
    let expires_at = now + AUTHENTICATED_TTL;
    let previous = match session
        .get::<super::session::Owner>(super::session::OWNER_KEY)
        .await
    {
        Ok(Some(previous)) => previous,
        Ok(None) => return callback_failure(FailureCode::AuthenticationFailed, return_to),
        Err(_) => return callback_failure(FailureCode::ProviderUnavailable, return_to),
    };
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, return_to);
    }
    let marker = super::session::Owner::fresh(previous.lineage);
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(marker.clone());
        authority.creating = true;
    }
    let saved = async {
        session.cycle_id().await?;
        session.clear().await;
        session.set_expiry(Some(Expiry::AtDateTime(expires_at)));
        session.insert(super::session::OWNER_KEY, marker).await?;
        session
            .insert(
                AUTHENTICATED_SESSION_KEY,
                AuthenticatedSession {
                    authenticated_at: now.unix_timestamp(),
                    expires_at: expires_at.unix_timestamp(),
                },
            )
            .await?;
        session.save().await
    }
    .await;
    if saved.is_err() || !store.current(session).await {
        let _ = store.revoke(lineage).await;
        // cycle_id has already deleted the old ID. Do not let middleware retry
        // a provisional create, including after a partial backing-store write.
        session.clear().await;
        let _ = session.flush().await;
        return callback_failure(FailureCode::ProviderUnavailable, return_to);
    }
    let mut response = redirect(return_to);
    response
        .extensions_mut()
        .insert(super::session::CallbackIntent::verified(return_to));
    response
}

async fn logout(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    Extension(super::session::OriginalSessionId(original_id)): Extension<
        super::session::OriginalSessionId,
    >,
    Extension(capture): Extension<super::session::ResponseCapture>,
) -> Response {
    capture.remember_logout();
    let store = &authentication.sessions.store;
    let lineage = original_id.and_then(|id| store.for_id(id));
    let mut failed;
    if let Some(lineage) = lineage {
        let _gate = lineage.gate.lock().await;
        let state = lineage
            .authority
            .lock()
            .await
            .committed
            .as_ref()
            .and_then(|record| record.data.get(TRANSACTION_SESSION_KEY))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        if let Some(state) = state {
            authentication
                .transactions
                .consume(&state, authentication.clock.now())
                .await;
        }
        failed = store.revoke(&lineage).await.is_err();
        // Flush the independently cached request too. Aliases cannot restore the
        // current record, and a failure still returns 500 with cookie expiry.
        failed |= session.flush().await.is_err();
    } else {
        failed = session.flush().await.is_err();
    }
    let mut response = if failed {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::empty())
            .expect("static response builds")
    } else {
        redirect("/login?logged_out=1")
    };
    response
        .extensions_mut()
        .insert(super::session::LogoutExpiry);
    response.headers_mut().append(
        header::SET_COOKIE,
        super::session::removal_cookie(authentication.config.secure_cookie()),
    );
    response
}

#[derive(Debug)]
enum CallbackQuery {
    Success { state: String, code: String },
    ProviderError { state: String },
}

impl CallbackQuery {
    fn parse(raw_query: Option<&str>) -> Option<Self> {
        let mut state = None;
        let mut code = None;
        let mut error = None;
        let mut error_description = None;
        let mut error_uri = None;
        let raw_query = raw_query?;
        for pair in raw_query.split('&') {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let key = decode_query_component(key)?;
            let value = decode_query_component(value)?;
            let target = match key.as_str() {
                "code" => &mut code,
                "state" => &mut state,
                "error" => &mut error,
                "error_description" => &mut error_description,
                "error_uri" => &mut error_uri,
                _ => continue,
            };
            if target.replace(value).is_some() {
                return None;
            }
        }
        let state = state.filter(|state| !state.is_empty())?;
        match (code, error, error_description, error_uri) {
            (Some(code), None, None, None) if !code.is_empty() => {
                Some(Self::Success { state, code })
            }
            (None, Some(error), _, _) if !error.is_empty() => Some(Self::ProviderError { state }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum FailureCode {
    AuthenticationFailed,
    ProviderUnavailable,
}

impl FailureCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::AuthenticationFailed => "authentication_failed",
            Self::ProviderUnavailable => "provider_unavailable",
        }
    }
}

fn callback_failure(error: FailureCode, return_to: &str) -> Response {
    let mut response = failure_redirect(error, return_to);
    response
        .extensions_mut()
        .insert(super::session::CallbackIntent::failed(return_to));
    response
}

pub(super) fn failure_redirect(error: FailureCode, return_to: &str) -> Response {
    let location = relative_url_with_query(
        "/login",
        &[("error", error.as_str()), ("return_to", return_to)],
    );
    redirect(&location)
}

pub(crate) fn redirect(location: &str) -> Response {
    match HeaderValue::from_str(location) {
        Ok(location) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, location)
            .body(axum::body::Body::empty())
            .expect("static redirect response should build"),
        Err(_) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, "/")
            .body(axum::body::Body::empty())
            .expect("fallback redirect response should build"),
    }
}

pub(crate) fn login_redirect(return_to: &str) -> Response {
    let location = relative_url_with_query("/login", &[("return_to", return_to)]);
    let mut response = redirect(&location);
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn relative_url_with_query(path: &str, pairs: &[(&str, &str)]) -> String {
    let mut url = url::Url::parse("http://localhost/").expect("static URL should parse");
    url.set_path(path);
    {
        let mut query = url.query_pairs_mut();
        for (key, value) in pairs {
            query.append_pair(key, value);
        }
    }
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_callback_vectors_validate_success_and_provider_error_shapes() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/auth/callbacks.json"))
                .expect("callback fixture should parse");
        for vector in fixture["vectors"].as_array().unwrap() {
            let kind = match CallbackQuery::parse(vector["query"].as_str()) {
                Some(CallbackQuery::Success { .. }) => "success",
                Some(CallbackQuery::ProviderError { .. }) => "error",
                None => "invalid",
            };
            assert_eq!(
                kind,
                vector["target"][super::super::app::FIXTURE_APP],
                "{}",
                vector["id"]
            );
        }
    }

    #[test]
    fn callback_shapes_decode_names_and_values_before_validation() {
        for query in [
            "error=denied",
            "error=denied&state=",
            "state=s&code",
            "error=&state=s&error_uri=detail",
            "error=denied&state=s&error_description=&%65rror_description=detail",
            "error=denied&state=s&error_uri=&%65rror_uri=detail",
            "code=c&state=s&%63ode=",
            "error=denied&state=s&%65rror=",
            "code=c&state=s&%GG=ignored",
            "code=c&state=s&extra=ok&extra=%FF",
        ] {
            assert!(CallbackQuery::parse(Some(query)).is_none(), "{query}");
        }
        assert!(matches!(
            CallbackQuery::parse(Some(
                "error=denied&state=s&error_description=&error_uri="
            )),
            Some(CallbackQuery::ProviderError { state }) if state == "s"
        ));
        assert!(matches!(
            CallbackQuery::parse(Some("%63ode=a%2Bb+c&%73tate=s%2Bt+u&extra=1&extra=2")),
            Some(CallbackQuery::Success { state, code }) if state == "s+t u" && code == "a+b c"
        ));
    }
}
