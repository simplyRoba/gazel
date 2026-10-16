use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use openidconnect::{CsrfToken, Nonce, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tower_sessions::{Expiry, Session};

use super::Authentication;
use super::runtime::ProtocolError;

pub(crate) const RETURN_TO_MAX_BYTES: usize = 2_048;
const TRANSACTION_TTL: Duration = Duration::minutes(5);
const AUTHENTICATED_TTL: Duration = Duration::hours(12);
const TRANSACTION_SESSION_KEY: &str = "oidc_transaction";
const AUTHENTICATED_SESSION_KEY: &str = "authenticated";

/// Time source used for transaction and authenticated-session expiry.
pub trait Clock: Send + Sync {
    /// Return the current UTC time.
    fn now(&self) -> OffsetDateTime;

    /// Return the current process-local monotonic time.
    #[doc(hidden)]
    fn monotonic_now(&self) -> Instant {
        Instant::now()
    }
}

#[derive(Debug)]
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

pub(crate) struct LoginTransaction {
    pub(crate) nonce: Nonce,
    pub(crate) pkce_verifier: PkceCodeVerifier,
    pub(crate) return_to: String,
    pub(crate) expires_at: OffsetDateTime,
}

impl fmt::Debug for LoginTransaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginTransaction")
            .field("nonce", &"[REDACTED]")
            .field("pkce_verifier", &"[REDACTED]")
            .field("return_to", &self.return_to)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AuthenticatedSession {
    authenticated_at: i64,
    expires_at: i64,
}

#[derive(Default)]
pub(crate) struct TransactionRegistry {
    entries: Mutex<HashMap<String, LoginTransaction>>,
}

impl fmt::Debug for TransactionRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransactionRegistry")
            .finish_non_exhaustive()
    }
}

impl TransactionRegistry {
    async fn replace(
        &self,
        old_state: Option<&str>,
        state: String,
        transaction: LoginTransaction,
        now: OffsetDateTime,
    ) {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, transaction| transaction.expires_at > now);
        if let Some(old_state) = old_state {
            entries.remove(old_state);
        }
        entries.insert(state, transaction);
    }

    async fn consume(&self, state: &str, now: OffsetDateTime) -> Option<LoginTransaction> {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, transaction| transaction.expires_at > now);
        entries.remove(state)
    }
}

pub(crate) fn routes(authentication: &Arc<Authentication>) -> Router {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .with_state(Arc::clone(authentication))
}

async fn login(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let return_to = return_to_from_query(raw_query.as_deref());
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
    authentication
        .transactions
        .replace(
            old_state.as_deref(),
            state_secret.clone(),
            LoginTransaction {
                nonce,
                pkce_verifier,
                return_to: return_to.clone(),
                expires_at,
            },
            now,
        )
        .await;

    session.set_expiry(Some(Expiry::AtDateTime(expires_at)));
    if session
        .insert(TRANSACTION_SESSION_KEY, state_secret)
        .await
        .is_err()
    {
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    }

    redirect(authorization_url.as_str())
}

async fn callback(
    State(authentication): State<Arc<Authentication>>,
    session: Session,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let Some(query) = CallbackQuery::parse(raw_query.as_deref()) else {
        return failure_redirect(FailureCode::AuthenticationFailed, "/");
    };
    let (returned_state, code) = match query {
        CallbackQuery::Success { state, code } => (state, Some(code)),
        CallbackQuery::ProviderError { state } => (state, None),
    };
    let bound_state = session
        .get::<String>(TRANSACTION_SESSION_KEY)
        .await
        .ok()
        .flatten();
    let state_matches = bound_state.is_some_and(|bound_state| {
        CsrfToken::new(bound_state) == CsrfToken::new(returned_state.clone())
    });
    if !state_matches {
        return failure_redirect(FailureCode::AuthenticationFailed, "/");
    }

    let Some(transaction) = authentication
        .transactions
        .consume(&returned_state, authentication.clock.now())
        .await
    else {
        return failure_redirect(FailureCode::AuthenticationFailed, "/");
    };
    let _ = session.remove::<String>(TRANSACTION_SESSION_KEY).await;

    let Some(code) = code else {
        return failure_redirect(FailureCode::AuthenticationFailed, &transaction.return_to);
    };

    match authentication
        .runtime
        .exchange_and_verify(code, &transaction.nonce, transaction.pkce_verifier)
        .await
    {
        Ok(_) => {}
        Err(ProtocolError::Authentication) => {
            return failure_redirect(FailureCode::AuthenticationFailed, &transaction.return_to);
        }
        Err(ProtocolError::ProviderUnavailable) => {
            return failure_redirect(FailureCode::ProviderUnavailable, &transaction.return_to);
        }
    }

    let now = authentication.clock.now();
    let expires_at = now + AUTHENTICATED_TTL;
    if session.cycle_id().await.is_err() {
        return failure_redirect(FailureCode::ProviderUnavailable, &transaction.return_to);
    }
    session.set_expiry(Some(Expiry::AtDateTime(expires_at)));
    if session
        .insert(
            AUTHENTICATED_SESSION_KEY,
            AuthenticatedSession {
                authenticated_at: now.unix_timestamp(),
                expires_at: expires_at.unix_timestamp(),
            },
        )
        .await
        .is_err()
    {
        return failure_redirect(FailureCode::ProviderUnavailable, &transaction.return_to);
    }

    redirect(&transaction.return_to)
}

impl Authentication {
    pub(crate) async fn is_authenticated(&self, session: &Session) -> bool {
        let record = session
            .get::<AuthenticatedSession>(AUTHENTICATED_SESSION_KEY)
            .await
            .ok()
            .flatten();
        match record {
            Some(record) if authenticated_session_is_valid(&record, self.clock.now()) => true,
            Some(_) => {
                let _ = session
                    .remove::<AuthenticatedSession>(AUTHENTICATED_SESSION_KEY)
                    .await;
                false
            }
            None => false,
        }
    }
}

fn authenticated_session_is_valid(record: &AuthenticatedSession, now: OffsetDateTime) -> bool {
    record.expires_at > now.unix_timestamp()
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
enum FailureCode {
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

fn failure_redirect(error: FailureCode, return_to: &str) -> Response {
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

pub(crate) fn validate_return_to(value: Option<&str>) -> String {
    let Some(value) = value else {
        return String::from("/");
    };
    if value.len() > RETURN_TO_MAX_BYTES || !value.starts_with('/') || value.starts_with("//") {
        return String::from("/");
    }

    let Some(decoded) = decode_percent_encoded(value, false) else {
        return String::from("/");
    };
    if decoded.contains('\\') || decoded.chars().any(char::is_control) {
        return String::from("/");
    }

    let path_end = value.find(['?', '#']).unwrap_or(value.len());
    let path = &value[..path_end];
    let Some(decoded_path) = decode_percent_encoded(path, false) else {
        return String::from("/");
    };
    if decoded_path.contains(['%', '?', '#'])
        || decoded_path.matches('/').count() != path.matches('/').count()
        || is_reserved_path(&decoded_path)
        || has_dot_segment(&decoded_path)
        || url::Url::parse(&format!("https://auth.invalid{value}")).is_err()
    {
        return String::from("/");
    }
    value.to_string()
}

fn has_dot_segment(path: &str) -> bool {
    path.split('/')
        .any(|segment| segment == "." || segment == "..")
}

pub(crate) fn login_redirect(return_to: &str) -> Response {
    let location = relative_url_with_query("/login", &[("return_to", return_to)]);
    redirect(&location)
}

fn is_reserved_path(path: &str) -> bool {
    ["/api", "/auth", "/health", "/login"]
        .iter()
        .any(|reserved| path == *reserved || path.starts_with(&format!("{reserved}/")))
}

fn return_to_from_query(raw_query: Option<&str>) -> String {
    let Some(raw_query) = raw_query else {
        return String::from("/");
    };
    for pair in raw_query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if decode_query_component(key).as_deref() == Some("return_to") {
            return validate_return_to(decode_query_component(value).as_deref());
        }
    }
    String::from("/")
}

fn decode_query_component(value: &str) -> Option<String> {
    decode_percent_encoded(value, true)
}

fn decode_percent_encoded(value: &str, plus_as_space: bool) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' if plus_as_space => decoded.push(b' '),
            b'%' => {
                let high = *bytes.get(index + 1)?;
                let low = *bytes.get(index + 2)?;
                decoded.push(hex(high)? << 4 | hex(low)?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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
    fn shared_return_vectors_follow_the_common_contract() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/auth/return-targets.json"
        ))
        .expect("return fixture should parse");
        for vector in fixture["vectors"].as_array().unwrap() {
            let input = &vector["input"];
            let value = if input.is_null() {
                None
            } else if let Some(value) = input.as_str() {
                Some(value.to_owned())
            } else {
                Some(format!(
                    "{}{}{}",
                    input["prefix"].as_str().unwrap(),
                    input["repeat"]
                        .as_str()
                        .unwrap()
                        .repeat(input["count"].as_u64().unwrap().try_into().unwrap()),
                    input["suffix"].as_str().unwrap()
                ))
            };
            let expected = if vector["target"]["gazel"].as_bool().unwrap() {
                value.as_deref().unwrap_or("/")
            } else {
                "/"
            };
            assert_eq!(
                validate_return_to(value.as_deref()),
                expected,
                "{}",
                vector["id"]
            );

            // Exercise exactly one form decoding, including wire size > serialized size.
            let wire = value.as_ref().map(|value| {
                url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("return_to", value)
                    .finish()
            });
            assert_eq!(
                return_to_from_query(wire.as_deref()),
                expected,
                "{}",
                vector["id"]
            );
        }
    }

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
            assert_eq!(kind, vector["target"]["gazel"], "{}", vector["id"]);
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

    fn transaction(expires_at: OffsetDateTime) -> LoginTransaction {
        LoginTransaction {
            nonce: Nonce::new(String::from("nonce")),
            pkce_verifier: PkceCodeVerifier::new("x".repeat(43)),
            return_to: String::from("/settings"),
            expires_at,
        }
    }

    #[test]
    fn return_target_validation_rejects_ambiguous_or_reserved_locations() {
        assert_eq!(
            validate_return_to(Some("/settings?tab=data#export")),
            "/settings?tab=data#export"
        );
        let encoded_boundary = format!("/{}", "%61".repeat(RETURN_TO_MAX_BYTES - 1));
        assert_eq!(validate_return_to(Some(&encoded_boundary)), "/");

        for unsafe_target in [
            "https://attacker.example/",
            "//attacker.example/",
            "%2Fsettings",
            "/x/../auth/login",
            "/x/%2e%2e/auth/login",
            "/./settings",
            "/\\attacker",
            "/foo%5Cbar",
            "/settings%0Aheader",
            "/%61pi/vehicles",
            "/api",
            "/auth/login",
            "/health",
            "/login",
        ] {
            assert_eq!(validate_return_to(Some(unsafe_target)), "/");
        }
        assert_eq!(
            validate_return_to(Some(&format!("/{}", "%61".repeat(RETURN_TO_MAX_BYTES)))),
            "/"
        );
        assert_eq!(validate_return_to(None), "/");
    }

    #[tokio::test]
    async fn transaction_registry_expires_at_exactly_five_minutes() {
        let registry = TransactionRegistry::default();
        let start = OffsetDateTime::UNIX_EPOCH;
        let expires_at = start + TRANSACTION_TTL;

        registry
            .replace(
                None,
                String::from("before-boundary"),
                transaction(expires_at),
                start,
            )
            .await;
        assert!(
            registry
                .consume("before-boundary", expires_at - Duration::nanoseconds(1))
                .await
                .is_some()
        );

        registry
            .replace(
                None,
                String::from("at-boundary"),
                transaction(expires_at),
                start,
            )
            .await;
        assert!(registry.consume("at-boundary", expires_at).await.is_none());
    }

    #[test]
    fn authenticated_session_expires_at_exactly_twelve_hours() {
        let start = OffsetDateTime::UNIX_EPOCH;
        let expires_at = start + AUTHENTICATED_TTL;
        let record = AuthenticatedSession {
            authenticated_at: start.unix_timestamp(),
            expires_at: expires_at.unix_timestamp(),
        };

        assert!(authenticated_session_is_valid(
            &record,
            expires_at - Duration::seconds(1)
        ));
        assert!(!authenticated_session_is_valid(&record, expires_at));
    }
}
