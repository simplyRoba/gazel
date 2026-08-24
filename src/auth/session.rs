use std::fmt;

use tower_sessions::cookie::{Key, SameSite};
use tower_sessions::service::PrivateCookie;
use tower_sessions::{MemoryStore, SessionManagerLayer};

const COOKIE_KEY_LENGTH: usize = 64;
pub const SESSION_COOKIE_NAME: &str = "gazel_session";

/// Concrete private-cookie session layer used by the authentication router.
pub type AuthSessionLayer = SessionManagerLayer<MemoryStore, PrivateCookie>;

/// Failure to obtain cryptographically secure randomness for the cookie key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CookieKeyError;

impl fmt::Display for CookieKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("failed to generate the authentication cookie key")
    }
}

impl std::error::Error for CookieKeyError {}

/// Per-process memory store and private-cookie key.
#[derive(Clone)]
pub struct SessionBackend {
    store: MemoryStore,
    key: Key,
    secure: bool,
}

impl fmt::Debug for SessionBackend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionBackend")
            .field("store", &self.store)
            .field("key", &"[REDACTED]")
            .field("secure", &self.secure)
            .finish()
    }
}

impl SessionBackend {
    /// Create a fresh process-local session backend.
    ///
    /// # Errors
    ///
    /// Returns an error when the operating system cannot provide secure
    /// randomness for the private-cookie key.
    pub fn new(secure: bool) -> Result<Self, CookieKeyError> {
        Ok(Self {
            store: MemoryStore::default(),
            key: generate_cookie_key()?,
            secure,
        })
    }

    /// Build the outer session middleware layer.
    pub fn layer(&self) -> AuthSessionLayer {
        SessionManagerLayer::new(self.store.clone())
            .with_name(SESSION_COOKIE_NAME)
            .with_http_only(true)
            .with_same_site(SameSite::Lax)
            .with_secure(self.secure)
            .with_path("/")
            .with_private(self.key.clone())
    }
}

fn generate_cookie_key() -> Result<Key, CookieKeyError> {
    generate_cookie_key_with(|bytes| getrandom::fill(bytes).map_err(|_| CookieKeyError))
}

fn generate_cookie_key_with(
    fill: impl FnOnce(&mut [u8]) -> Result<(), CookieKeyError>,
) -> Result<Key, CookieKeyError> {
    let mut bytes = [0_u8; COOKIE_KEY_LENGTH];
    fill(&mut bytes)?;
    Key::try_from(bytes.as_slice()).map_err(|_| CookieKeyError)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::get;
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session};

    use super::*;

    const AUTHENTICATED_KEY: &str = "authenticated";

    #[test]
    fn generated_cookie_keys_are_process_unique() {
        let first = generate_cookie_key().expect("secure randomness should be available");
        let second = generate_cookie_key().expect("secure randomness should be available");

        assert_ne!(first.master(), second.master());
        assert_eq!(first.master().len(), COOKIE_KEY_LENGTH);
    }

    #[test]
    fn cookie_key_generation_failure_is_reported() {
        let result = generate_cookie_key_with(|_| Err(CookieKeyError));
        assert_eq!(result.expect_err("generation should fail"), CookieKeyError);
    }

    #[tokio::test]
    async fn replacing_key_or_store_invalidates_an_existing_cookie() {
        let store = MemoryStore::default();
        let first_key = Key::from(&[1_u8; COOKIE_KEY_LENGTH]);
        let replacement_key = Key::from(&[2_u8; COOKIE_KEY_LENGTH]);

        let response = test_router(store.clone(), first_key.clone())
            .oneshot(
                Request::builder()
                    .uri("/issue")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .expect("session cookie should be set")
            .to_str()
            .expect("cookie should be ASCII")
            .split(';')
            .next()
            .expect("cookie pair should be present")
            .to_string();

        assert_eq!(
            check_cookie(store.clone(), first_key.clone(), &cookie).await,
            StatusCode::OK
        );
        assert_eq!(
            check_cookie(store, replacement_key, &cookie).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            check_cookie(MemoryStore::default(), first_key, &cookie).await,
            StatusCode::UNAUTHORIZED
        );
    }

    fn test_router(store: MemoryStore, key: Key) -> Router {
        let layer = SessionManagerLayer::new(store)
            .with_name(SESSION_COOKIE_NAME)
            .with_secure(false)
            .with_same_site(SameSite::Lax)
            .with_private(key);

        Router::new()
            .route("/issue", get(issue_session))
            .route("/check", get(check_session))
            .layer(layer)
    }

    async fn issue_session(session: Session) -> StatusCode {
        session
            .insert(AUTHENTICATED_KEY, true)
            .await
            .expect("session write should succeed");
        StatusCode::NO_CONTENT
    }

    async fn check_session(session: Session) -> StatusCode {
        match session
            .get::<bool>(AUTHENTICATED_KEY)
            .await
            .expect("session read should succeed")
        {
            Some(true) => StatusCode::OK,
            _ => StatusCode::UNAUTHORIZED,
        }
    }

    async fn check_cookie(store: MemoryStore, key: Key, cookie: &str) -> StatusCode {
        test_router(store, key)
            .oneshot(
                Request::builder()
                    .uri("/check")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed")
            .status()
    }
}
