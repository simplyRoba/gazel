use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, Mutex as IndexMutex};
use std::task::{Context, Poll};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, header};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tokio::sync::{Mutex as AsyncMutex, Semaphore};
use tower::{Layer, Service};
use tower_sessions::cookie::{Key, SameSite};
use tower_sessions::service::PrivateCookie;
use tower_sessions::session::{Id, Record};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer, SessionStore, session_store};

use super::app::SESSION_COOKIE_NAME;
use super::{Authentication, Clock};

const COOKIE_KEY_LENGTH: usize = 64;

/// Concrete private-cookie session layer used by the authentication router.
#[derive(Clone, Debug)]
pub struct AuthSessionLayer {
    manager: SessionManagerLayer<AuthStore, PrivateCookie>,
    store: AuthStore,
    secure: bool,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct Handoff<S>(S);

#[doc(hidden)]
#[derive(Clone)]
pub struct AuthSessionService<S> {
    inner: S,
    store: AuthStore,
    secure: bool,
}

type SessionCapture = Arc<Mutex<CapturedResponse>>;

#[derive(Clone)]
pub(super) struct ResponseCapture(SessionCapture);

impl ResponseCapture {
    pub(super) fn remember_callback(&self, intent: CallbackIntent) {
        self.0.lock().expect("session capture poisoned").callback = Some(intent);
    }

    pub(super) fn remember_logout(&self) {
        self.0.lock().expect("session capture poisoned").logout = true;
    }
}

#[derive(Clone, Default)]
struct CapturedResponse {
    session: Option<Session>,
    original_id: Option<tower_sessions::session::Id>,
    callback: Option<CallbackIntent>,
    logout: bool,
    unrelated_cookies: Vec<axum::http::HeaderValue>,
}

/// Only constructed from a validated requested/transaction target, never from
/// the provider Location header or callback query.
#[derive(Clone)]
pub(super) struct CallbackIntent {
    target: String,
    verified: bool,
}

impl CallbackIntent {
    pub(super) fn failed(target: &str) -> Self {
        Self {
            target: super::return_to::validate_return_to(Some(target)),
            verified: false,
        }
    }

    pub(super) fn verified(target: &str) -> Self {
        Self {
            verified: true,
            ..Self::failed(target)
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct OriginalSessionId(pub Option<tower_sessions::session::Id>);

#[derive(Clone)]
pub(super) struct LogoutExpiry;

impl<S> Layer<S> for AuthSessionLayer {
    type Service = AuthSessionService<
        <SessionManagerLayer<AuthStore, PrivateCookie> as Layer<Handoff<S>>>::Service,
    >;

    fn layer(&self, inner: S) -> Self::Service {
        AuthSessionService {
            inner: self.manager.layer(Handoff(inner)),
            store: self.store.clone(),
            secure: self.secure,
        }
    }
}

impl<S> Service<Request<Body>> for Handoff<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let session = request.extensions().get::<Session>().cloned();
        request
            .extensions_mut()
            .insert(OriginalSessionId(session.as_ref().and_then(Session::id)));
        let capture = request.extensions().get::<SessionCapture>().cloned();
        if let Some(capture) = &capture {
            request
                .extensions_mut()
                .insert(ResponseCapture(Arc::clone(capture)));
            let mut capture = capture.lock().expect("session capture poisoned");
            capture.original_id = session.as_ref().and_then(Session::id);
            capture.session.clone_from(&session);
        }
        let future = self.0.call(request);
        Box::pin(async move {
            let mut response = future.await?;
            if let Some(capture) = capture {
                let mut capture = capture.lock().expect("session capture poisoned");
                if let Some(intent) = response.extensions().get::<CallbackIntent>() {
                    capture.callback = Some(intent.clone());
                }
                capture.logout |= response.extensions().get::<LogoutExpiry>().is_some();
                capture.unrelated_cookies = response
                    .headers()
                    .get_all(header::SET_COOKIE)
                    .iter()
                    .filter(|cookie| !is_session_cookie(cookie))
                    .cloned()
                    .collect();
            }
            if let Some(session) = session {
                response.extensions_mut().insert(session);
            }
            Ok(response)
        })
    }
}

impl<S> Service<Request<Body>> for AuthSessionService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let capture = SessionCapture::default();
        request.extensions_mut().insert(Arc::clone(&capture));
        let future = self.inner.call(request);
        let store = self.store.clone();
        let secure = self.secure;
        Box::pin(async move {
            let mut response = future.await?;
            let captured = capture.lock().expect("session capture poisoned").clone();
            finalize_response(&mut response, &store, captured, secure).await;
            Ok(response)
        })
    }
}

async fn finalize_response(
    response: &mut Response,
    store: &AuthStore,
    captured: CapturedResponse,
    secure: bool,
) {
    let metadata = response.extensions_mut().remove::<Session>();
    let missing = metadata.is_none();
    let session = metadata.or(captured.session);
    let lineage = session
        .as_ref()
        .and_then(Session::id)
        .or(captured.original_id)
        .and_then(|id| store.for_id(id));
    // Keep the ownership check and synchronous header decision in the same
    // lifecycle critical section as replacement/logout, not a cached bool.
    let _gate = match &lineage {
        Some(lineage) => Some(lineage.gate.lock().await),
        None => None,
    };
    let mut current = match &session {
        Some(session) => store.current(session).await,
        None => false,
    };
    let callback_failed =
        captured.callback.is_some() && (missing || response.status().is_server_error());
    if callback_failed
        && current
        && captured
            .callback
            .as_ref()
            .is_some_and(|intent| intent.verified)
        && let Some(lineage) = &lineage
    {
        let _ = store.revoke(lineage).await;
        current = false;
    }
    let removal_allowed = captured.logout
        || match &lineage {
            Some(lineage) => lineage.authority.lock().await.committed.is_none(),
            None => true,
        };
    if missing || callback_failed {
        if let Some(intent) = captured.callback {
            // The manager discarded the handler response. Preserve the callback
            // contract and its trusted target even when extensions were lost.
            *response = super::routes::failure_redirect(
                super::routes::FailureCode::ProviderUnavailable,
                &intent.target,
            );
            if current || removal_allowed {
                response
                    .headers_mut()
                    .append(header::SET_COOKIE, removal_cookie(secure));
            }
        } else if captured.logout || !current && removal_allowed {
            response
                .headers_mut()
                .append(header::SET_COOKIE, removal_cookie(secure));
        }
        for cookie in captured.unrelated_cookies {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
    }
    if captured.logout
        && !response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .any(|cookie| {
                cookie
                    .to_str()
                    .is_ok_and(|value| value.starts_with(&format!("{SESSION_COOKIE_NAME}=;")))
            })
    {
        response
            .headers_mut()
            .append(header::SET_COOKIE, removal_cookie(secure));
    }
    filter_cookies(response, current && !missing, removal_allowed);
}

fn is_session_cookie(value: &axum::http::HeaderValue) -> bool {
    value
        .to_str()
        .is_ok_and(|value| value.starts_with(&format!("{SESSION_COOKIE_NAME}=")))
}

fn filter_cookies(response: &mut Response, current: bool, removal_allowed: bool) {
    let cookies: Vec<_> = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .cloned()
        .collect();
    response.headers_mut().remove(header::SET_COOKIE);
    let retained: Vec<_> = cookies
        .iter()
        .filter(|value| {
            let Ok(value) = value.to_str() else {
                return true;
            };
            if !value.starts_with(&format!("{SESSION_COOKIE_NAME}=")) {
                return true;
            }
            // Explicit logout expiry and empty-session removals are retained. A stale
            // nonempty session's cookie may not overwrite the winning generation.
            current || (removal_allowed && value.starts_with(&format!("{SESSION_COOKIE_NAME}=;")))
        })
        .cloned()
        .collect();
    for value in retained {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

pub(super) fn removal_cookie(secure: bool) -> axum::http::HeaderValue {
    tower_sessions::cookie::Cookie::build((SESSION_COOKIE_NAME, ""))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::ZERO)
        .expires(time::OffsetDateTime::UNIX_EPOCH)
        .build()
        .to_string()
        .parse()
        .expect("static removal cookie is valid")
}

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
    pub(super) store: AuthStore,
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
    pub fn new(
        secure: bool,
        clock: Arc<dyn Clock>,
        faults: SessionFaults,
    ) -> Result<Self, CookieKeyError> {
        Ok(Self {
            store: AuthStore::new(clock, faults),
            key: generate_cookie_key()?,
            secure,
        })
    }

    /// Build the outer session middleware layer.
    pub fn layer(&self) -> AuthSessionLayer {
        let manager = SessionManagerLayer::new(self.store.clone())
            .with_name(SESSION_COOKIE_NAME)
            .with_http_only(true)
            .with_same_site(SameSite::Lax)
            .with_secure(self.secure)
            .with_path("/")
            .with_private(self.key.clone());
        AuthSessionLayer {
            manager,
            store: self.store.clone(),
            secure: self.secure,
        }
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

    async fn owned_session(backend: &SessionBackend) -> (Session, Arc<super::Lineage>) {
        use super::{OWNER_KEY, Owner};
        let expiry = time::OffsetDateTime::now_utc() + time::Duration::minutes(5);
        let (key, lineage) = backend.store.allocate(expiry);
        let marker = Owner::fresh(key);
        let session = Session::new(
            None,
            Arc::new(backend.store.clone()),
            Some(tower_sessions::Expiry::AtDateTime(expiry)),
        );
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = Some(marker.clone());
            authority.creating = true;
        }
        session.insert(OWNER_KEY, marker).await.unwrap();
        session.insert(AUTHENTICATED_KEY, true).await.unwrap();
        session.save().await.unwrap();
        (session, lineage)
    }

    enum FinalizerChange {
        Replacement,
        Revocation,
    }

    async fn assert_finalizer_rechecks_after_gate(change: FinalizerChange) {
        use super::{OWNER_KEY, Owner};

        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::transaction::SystemClock),
            SessionFaults::default(),
        )
        .unwrap();
        let (winner, lineage) = owned_session(&backend).await;
        // Session::new is essential: cloning winner would share its cached R+1.
        let stale = Session::new(
            winner.id(),
            Arc::new(backend.store.clone()),
            winner.expiry(),
        );
        let previous = stale.get::<Owner>(OWNER_KEY).await.unwrap().unwrap();
        assert!(backend.store.current(&stale).await);
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!("{SESSION_COOKIE_NAME}=obsolete; Path=/"),
            )
            .header(header::SET_COOKIE, "unrelated=kept; Path=/")
            .body(Body::empty())
            .unwrap();
        response.extensions_mut().insert(stale.clone());

        let gate = lineage.gate.lock().await;
        let (entered, entry) = tokio::sync::oneshot::channel();
        let store = backend.store.clone();
        let task = tokio::spawn(async move {
            entered.send(()).unwrap();
            finalize_response(&mut response, &store, CapturedResponse::default(), false).await;
            response
        });
        entry.await.unwrap();
        // On the current-thread runtime the task has reached its first pending
        // await. A finalizer without the lifecycle gate can already finish here.
        tokio::task::yield_now().await;
        assert!(
            !task.is_finished(),
            "finalization must wait for the lineage gate"
        );

        match change {
            FinalizerChange::Replacement => {
                let marker = Owner::fresh(previous.lineage);
                lineage.authority.lock().await.permit = Some(marker.clone());
                winner.insert(OWNER_KEY, marker.clone()).await.unwrap();
                winner.save().await.unwrap();
                assert_eq!(winner.get::<Owner>(OWNER_KEY).await.unwrap(), Some(marker));
                assert!(backend.store.current(&winner).await);
            }
            FinalizerChange::Revocation => backend.store.revoke(&lineage).await.unwrap(),
        }
        // The obsolete request still has its own cached R after the mutation.
        assert!(!backend.store.current(&stale).await);
        drop(gate);
        let response = task.await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookies: Vec<_> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .collect();
        assert_eq!(
            cookies,
            vec![&axum::http::HeaderValue::from_static(
                "unrelated=kept; Path=/"
            )]
        );
        match change {
            FinalizerChange::Replacement => assert!(backend.store.current(&winner).await),
            FinalizerChange::Revocation => {
                assert!(!backend.store.current(&winner).await);
                assert!(lineage.authority.lock().await.committed.is_none());
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ordinary_finalizer_waits_for_gate_and_rechecks_replacement() {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            assert_finalizer_rechecks_after_gate(FinalizerChange::Replacement),
        )
        .await
        .expect("finalizer replacement race must not stall");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ordinary_finalizer_waits_for_gate_and_rechecks_revocation() {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            assert_finalizer_rechecks_after_gate(FinalizerChange::Revocation),
        )
        .await
        .expect("finalizer revocation race must not stall");
    }

    fn private_cookie(backend: &SessionBackend, id: tower_sessions::session::Id) -> String {
        let mut jar = tower_sessions::cookie::CookieJar::new();
        jar.private_mut(&backend.key)
            .add(tower_sessions::cookie::Cookie::new(
                SESSION_COOKIE_NAME,
                id.to_string(),
            ));
        jar.get(SESSION_COOKIE_NAME).unwrap().to_string()
    }

    async fn authoritative_check(
        axum::extract::State(store): axum::extract::State<AuthStore>,
        session: Session,
    ) -> StatusCode {
        if session.get::<bool>(AUTHENTICATED_KEY).await.unwrap() == Some(true)
            && store.current(&session).await
        {
            StatusCode::OK
        } else {
            StatusCode::UNAUTHORIZED
        }
    }

    #[tokio::test]
    async fn lost_callback_metadata_revokes_both_old_and_provisional_private_cookies() {
        use super::{OWNER_KEY, Owner};
        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::transaction::SystemClock),
            SessionFaults::default(),
        )
        .unwrap();
        let (session, lineage) = owned_session(&backend).await;
        let old_id = session.id().unwrap();
        let previous = session.get::<Owner>(OWNER_KEY).await.unwrap().unwrap();
        let marker = Owner::fresh(previous.lineage);
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = Some(marker.clone());
            authority.creating = true;
        }
        session.cycle_id().await.unwrap();
        session.insert(OWNER_KEY, marker).await.unwrap();
        session.save().await.unwrap();
        let proposed_id = session.id().unwrap();
        let mut response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap();
        finalize_response(
            &mut response,
            &backend.store,
            CapturedResponse {
                session: Some(session),
                callback: Some(CallbackIntent::verified("/settings?tab=data#export")),
                ..Default::default()
            },
            false,
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers()[header::LOCATION],
            "/login?error=provider_unavailable&return_to=%2Fsettings%3Ftab%3Ddata%23export"
        );
        assert!(
            response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains("Max-Age=0")
        );
        let app = Router::new()
            .route("/check", get(authoritative_check))
            .with_state(backend.store.clone())
            .layer(backend.layer());
        for id in [old_id, proposed_id] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/check")
                        .header(header::COOKIE, private_cookie(&backend, id))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn lost_stale_callback_metadata_cannot_revoke_a_newer_revision() {
        use super::{OWNER_KEY, Owner};
        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::transaction::SystemClock),
            SessionFaults::default(),
        )
        .unwrap();
        let (winner, lineage) = owned_session(&backend).await;
        let stale = Session::new(
            winner.id(),
            Arc::new(backend.store.clone()),
            winner.expiry(),
        );
        let previous = stale.get::<Owner>(OWNER_KEY).await.unwrap().unwrap();
        let marker = Owner::fresh(previous.lineage);
        lineage.authority.lock().await.permit = Some(marker.clone());
        winner.insert(OWNER_KEY, marker).await.unwrap();
        winner.save().await.unwrap();
        let mut response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap();
        finalize_response(
            &mut response,
            &backend.store,
            CapturedResponse {
                session: Some(stale),
                callback: Some(CallbackIntent::verified("/old")),
                ..Default::default()
            },
            false,
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers()[header::LOCATION],
            "/login?error=provider_unavailable&return_to=%2Fold"
        );
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        assert!(backend.store.current(&winner).await);
    }

    #[tokio::test]
    async fn unrelated_readonly_500_does_not_revoke_or_expire_a_valid_session() {
        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::transaction::SystemClock),
            SessionFaults::default(),
        )
        .unwrap();
        let (session, _) = owned_session(&backend).await;
        let cookie = private_cookie(&backend, session.id().unwrap());
        let app = Router::new()
            .route("/check", get(authoritative_check))
            .route(
                "/failure",
                get(|session: Session| async move {
                    assert_eq!(
                        session.get::<bool>(AUTHENTICATED_KEY).await.unwrap(),
                        Some(true)
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }),
            )
            .with_state(backend.store.clone())
            .layer(backend.layer());
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/failure")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        let check = app
            .oneshot(
                Request::builder()
                    .uri("/check")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(check.status(), StatusCode::OK);
        // Even an operational response replacement without callback provenance
        // must not turn a read-only API error into a logout.
        let mut response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap();
        finalize_response(
            &mut response,
            &backend.store,
            CapturedResponse {
                session: Some(session.clone()),
                ..Default::default()
            },
            false,
        )
        .await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        assert!(backend.store.current(&session).await);
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

pub(super) const AUTHENTICATED_SESSION_KEY: &str = "authenticated";
pub(super) const AUTHENTICATED_TTL: Duration = Duration::hours(12);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct AuthenticatedSession {
    pub(super) authenticated_at: i64,
    pub(super) expires_at: i64,
}

impl Authentication {
    pub(crate) async fn is_authenticated(&self, session: &Session) -> bool {
        let record = session
            .get::<AuthenticatedSession>(AUTHENTICATED_SESSION_KEY)
            .await
            .ok()
            .flatten();
        match record {
            Some(record) if authenticated_session_is_valid(&record, self.clock.now()) => {
                self.sessions.store.current(session).await
            }
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

pub(super) const OWNER_KEY: &str = "auth_owner";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct Owner {
    pub lineage: String,
    pub revision: String,
}

impl Owner {
    pub fn fresh(lineage: String) -> Self {
        Self {
            lineage,
            revision: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Authority {
    pub committed: Option<Record>,
    pub permit: Option<Owner>,
    pub creating: bool,
}

#[derive(Debug)]
pub(super) struct Lineage {
    // Protocol coordination and store serialization are distinct: handlers can save
    // while holding the lifecycle gate; the store never acquires it in reverse.
    pub gate: AsyncMutex<()>,
    pub authority: AsyncMutex<Authority>,
}

#[derive(Default)]
struct Index {
    lineages: HashMap<String, (Arc<Lineage>, OffsetDateTime)>,
    aliases: HashMap<Id, (String, OffsetDateTime)>,
}

/// Per-instance deterministic storage fault seam. No process environment is used.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOperation {
    Create,
    Save,
    Load,
    Delete,
}

/// A one-shot failure, optionally after the backing write has happened.
#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub struct SessionFaults {
    failure: Arc<IndexMutex<Option<(StoreOperation, bool)>>>,
    pause: Arc<IndexMutex<Option<(StoreOperation, SessionBarrier)>>>,
}

/// A deterministic, one-shot store barrier, scoped to one backend instance.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct SessionBarrier {
    entered: Arc<Semaphore>,
    released: Arc<Semaphore>,
}

impl SessionBarrier {
    pub async fn wait_entered(&self) {
        if let Ok(permit) = self.entered.acquire().await {
            permit.forget();
        }
    }

    pub fn release(&self) {
        self.released.add_permits(1);
    }
}

impl SessionFaults {
    /// # Panics
    /// Panics if an earlier test poisoned the fault mutex.
    pub fn fail_next(&self, operation: StoreOperation, after_write: bool) {
        *self.failure.lock().expect("fault mutex poisoned") = Some((operation, after_write));
    }

    /// # Panics
    /// Panics if an earlier test poisoned the fault mutex.
    pub fn pause_next(&self, operation: StoreOperation) -> SessionBarrier {
        let barrier = SessionBarrier {
            entered: Arc::new(Semaphore::new(0)),
            released: Arc::new(Semaphore::new(0)),
        };
        *self.pause.lock().expect("fault mutex poisoned") = Some((operation, barrier.clone()));
        barrier
    }

    async fn checkpoint(&self, operation: StoreOperation) -> Option<bool> {
        let fault = {
            let mut fault = self.failure.lock().expect("fault mutex poisoned");
            if fault.as_ref().is_some_and(|(op, _)| *op == operation) {
                fault.take().map(|(_, after)| after)
            } else {
                None
            }
        };
        let pause = {
            let mut pause = self.pause.lock().expect("fault mutex poisoned");
            if pause.as_ref().is_some_and(|(op, _)| *op == operation) {
                pause.take().map(|(_, barrier)| barrier)
            } else {
                None
            }
        };
        if let Some(barrier) = pause {
            barrier.entered.add_permits(1);
            barrier
                .released
                .acquire()
                .await
                .expect("barrier remains open")
                .forget();
        }
        fault
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct AuthStore {
    backing: MemoryStore,
    index: Arc<IndexMutex<Index>>,
    clock: Arc<dyn Clock>,
    pub faults: SessionFaults,
}

impl std::fmt::Debug for AuthStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthStore").finish_non_exhaustive()
    }
}

fn owner(record: &Record) -> Option<Owner> {
    serde_json::from_value(record.data.get(OWNER_KEY)?.clone()).ok()
}

fn fault_error() -> session_store::Error {
    session_store::Error::Backend("injected session persistence failure".into())
}

impl AuthStore {
    pub(super) fn new(clock: Arc<dyn Clock>, faults: SessionFaults) -> Self {
        Self {
            backing: MemoryStore::default(),
            index: Arc::default(),
            clock,
            faults,
        }
    }

    fn prune(index: &mut Index, now: OffsetDateTime) {
        index.aliases.retain(|_, (_, expiry)| *expiry > now);
        index.lineages.retain(|_, (_, expiry)| *expiry > now);
    }

    pub(super) fn lineage(&self, key: &str) -> Option<Arc<Lineage>> {
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        index
            .lineages
            .get(key)
            .map(|(lineage, _)| Arc::clone(lineage))
    }

    pub(super) fn for_id(&self, id: Id) -> Option<Arc<Lineage>> {
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        let (key, _) = index.aliases.get(&id)?;
        index
            .lineages
            .get(key)
            .map(|(lineage, _)| Arc::clone(lineage))
    }

    pub(super) fn allocate(&self, expiry: OffsetDateTime) -> (String, Arc<Lineage>) {
        let key = uuid::Uuid::new_v4().to_string();
        let lineage = Arc::new(Lineage {
            gate: AsyncMutex::new(()),
            authority: AsyncMutex::new(Authority::default()),
        });
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        index
            .lineages
            .insert(key.clone(), (Arc::clone(&lineage), expiry));
        (key, lineage)
    }

    pub(super) fn retire_empty(&self, key: &str) {
        // Called only for an allocation which never acquired a backing record.
        self.index
            .lock()
            .expect("session index poisoned")
            .lineages
            .remove(key);
    }

    fn remember(&self, record: &Record, owner: &Owner) {
        let mut index = self.index.lock().expect("session index poisoned");
        // Aliases have the same absolute retirement deadline as the current
        // generation. Rotation never creates permanent tombstones.
        for (key, expiry) in index.aliases.values_mut() {
            if *key == owner.lineage {
                *expiry = record.expiry_date;
            }
        }
        index
            .aliases
            .insert(record.id, (owner.lineage.clone(), record.expiry_date));
        if let Some((_, expiry)) = index.lineages.get_mut(&owner.lineage) {
            *expiry = record.expiry_date;
        }
    }

    pub(super) async fn current(&self, session: &Session) -> bool {
        let Some(id) = session.id() else {
            return false;
        };
        let Ok(Some(marker)) = session.get::<Owner>(OWNER_KEY).await else {
            return false;
        };
        self.owns(id, &marker).await
    }

    pub(super) async fn owns(&self, id: Id, marker: &Owner) -> bool {
        let Some(lineage) = self.lineage(&marker.lineage) else {
            return false;
        };
        let authority = lineage.authority.lock().await;
        authority.committed.as_ref().is_some_and(|record| {
            record.id == id
                && owner(record).as_ref() == Some(marker)
                && record.expiry_date > self.clock.now()
        }) && self.backing.load(&id).await.ok().flatten().is_some()
    }

    pub(super) async fn rollback_binding(&self, session: &Session, lineage: &Lineage, key: &str) {
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = authority.committed.as_ref().and_then(owner);
            authority.creating = false;
        }
        // Caller holds the lineage gate. Reload the old committed binding and
        // its absolute expiry; never roll back using the request's old snapshot.
        session.clear().await;
        if session.id().is_some() {
            let _ = self.refresh(session).await;
        }
        if lineage.authority.lock().await.committed.is_none() {
            self.retire_empty(key);
        }
    }

    pub(super) async fn revoke(&self, lineage: &Lineage) -> session_store::Result<()> {
        let mut authority = lineage.authority.lock().await;
        authority.permit = None;
        authority.creating = false;
        let record = authority.committed.take();
        if let Some(record) = record {
            // Keep terminal authority and rotation aliases until their existing
            // absolute deadline. Delayed requests must not become fresh logins.
            self.delete_backing(&record.id).await?;
        }
        Ok(())
    }

    async fn delete_backing(&self, id: &Id) -> session_store::Result<()> {
        let fault = self.faults.checkpoint(StoreOperation::Delete).await;
        if fault == Some(false) {
            return Err(fault_error());
        }
        self.backing.delete(id).await?;
        if fault == Some(true) {
            return Err(fault_error());
        }
        Ok(())
    }

    pub(super) async fn refresh(
        &self,
        session: &Session,
    ) -> Result<(), tower_sessions::session::Error> {
        session.load().await?;
        if let Some(lineage) = session.id().and_then(|id| self.for_id(id)) {
            let authority = lineage.authority.lock().await;
            if let Some(record) = authority
                .committed
                .as_ref()
                .filter(|record| Some(record.id) == session.id())
                && session.is_modified()
            {
                session.set_expiry(Some(tower_sessions::Expiry::AtDateTime(record.expiry_date)));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SessionStore for AuthStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        self.write(record, true).await
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.write(&mut record.clone(), false).await
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        if self.faults.checkpoint(StoreOperation::Load).await.is_some() {
            return Err(fault_error());
        }
        let Some(lineage) = self.for_id(*id) else {
            return Ok(None);
        };
        let authority = lineage.authority.lock().await;
        if !authority
            .committed
            .as_ref()
            .is_some_and(|record| record.id == *id)
        {
            return Ok(None);
        }
        self.backing.load(id).await
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        if let Some(lineage) = self.for_id(*id) {
            let _authority = lineage.authority.lock().await;
            self.delete_backing(id).await
        } else {
            self.delete_backing(id).await
        }
    }
}

impl AuthStore {
    async fn write(&self, record: &mut Record, create: bool) -> session_store::Result<()> {
        let operation = if create {
            StoreOperation::Create
        } else {
            StoreOperation::Save
        };
        // Pause before validation so a delayed middleware save can lose ownership
        // while another request progresses. Validation and backing I/O stay atomic.
        let fault = self.faults.checkpoint(operation).await;
        let Some(marker) = owner(record) else {
            return Ok(());
        };
        let Some(lineage) = self.lineage(&marker.lineage) else {
            return Ok(());
        };
        let mut authority = lineage.authority.lock().await;
        if authority.permit.as_ref() != Some(&marker) {
            return Ok(());
        }
        if create {
            if !authority.creating {
                return Ok(());
            }
        } else if !authority
            .committed
            .as_ref()
            .is_some_and(|old| old.id == record.id)
            || self.backing.load(&record.id).await?.is_none()
        {
            // MemoryStore::save upserts; never let a deleted alias reach it.
            return Ok(());
        }
        if !create
            && authority.committed.as_ref().is_some_and(|old| {
                old.id == record.id
                    && old.data == record.data
                    && old.expiry_date == record.expiry_date
            })
        {
            // The manager auto-saves explicitly persisted sessions. Existence was
            // checked above; an exact duplicate is not another backing write.
            return Ok(());
        }
        if fault == Some(false) {
            return Err(fault_error());
        }
        if create {
            self.backing.create(record).await?;
        } else {
            self.backing.save(record).await?;
        }
        if fault == Some(true) {
            if create {
                self.backing.delete(&record.id).await?;
            } else if let Some(old) = &authority.committed {
                self.backing.save(old).await?;
            }
            return Err(fault_error());
        }
        authority.committed = Some(record.clone());
        authority.creating = false;
        self.remember(record, &marker);
        Ok(())
    }
}

#[cfg(test)]
mod store_tests {
    use std::sync::Arc;

    use time::{Duration, OffsetDateTime};
    use tower_sessions::{Expiry, Session, SessionStore};

    use super::super::transaction::SystemClock;
    use super::Clock;
    use super::{
        AUTHENTICATED_TTL, AuthStore, AuthenticatedSession, Lineage, OWNER_KEY, Owner,
        SessionFaults, StoreOperation, authenticated_session_is_valid,
    };

    async fn seed(store: &AuthStore) -> (Session, Arc<Lineage>, Owner) {
        let expiry = OffsetDateTime::now_utc() + Duration::minutes(5);
        let (key, lineage) = store.allocate(expiry);
        let marker = Owner::fresh(key);
        let session = Session::new(
            None,
            Arc::new(store.clone()),
            Some(Expiry::AtDateTime(expiry)),
        );
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = Some(marker.clone());
            authority.creating = true;
        }
        session.insert(OWNER_KEY, &marker).await.unwrap();
        session.insert("oidc_transaction", "prior").await.unwrap();
        session.save().await.unwrap();
        (session, lineage, marker)
    }

    fn independent(store: &AuthStore, original: &Session) -> Session {
        Session::new(original.id(), Arc::new(store.clone()), None)
    }

    async fn rebind(session: &Session, lineage: &Lineage, marker: Owner, state: &str) {
        lineage.authority.lock().await.permit = Some(marker.clone());
        session.insert(OWNER_KEY, marker).await.unwrap();
        session.insert("oidc_transaction", state).await.unwrap();
        session.save().await.unwrap();
    }

    #[tokio::test]
    async fn independent_preloaded_snapshots_cannot_overwrite_a_newer_binding() {
        let faults = SessionFaults::default();
        let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
        let (original, lineage, marker) = seed(&store).await;
        let older = independent(&store, &original);
        let newer = independent(&store, &original);
        for session in [&older, &newer] {
            assert_eq!(
                session
                    .get::<String>("oidc_transaction")
                    .await
                    .unwrap()
                    .as_deref(),
                Some("prior")
            );
        }
        let pause = faults.pause_next(StoreOperation::Save);
        faults.fail_next(StoreOperation::Save, true);
        let task = tokio::spawn(async move { older.save().await });
        pause.wait_entered().await;
        let next = Owner::fresh(marker.lineage);
        rebind(&newer, &lineage, next, "winner").await;
        pause.release();
        task.await.unwrap().unwrap();
        assert!(store.current(&newer).await);
        let loaded = independent(&store, &original);
        assert_eq!(
            loaded
                .get::<String>("oidc_transaction")
                .await
                .unwrap()
                .as_deref(),
            Some("winner")
        );
    }

    #[tokio::test]
    async fn authoritative_reload_restores_data_and_absolute_expiry_before_mutation() {
        let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
        let (original, lineage, marker) = seed(&store).await;
        let stale = independent(&store, &original);
        stale.load().await.unwrap();
        stale.insert("outdated", true).await.unwrap();
        let next = Owner::fresh(marker.lineage);
        rebind(&original, &lineage, next.clone(), "winner").await;
        let expiry = lineage
            .authority
            .lock()
            .await
            .committed
            .as_ref()
            .unwrap()
            .expiry_date;
        store.refresh(&stale).await.unwrap();
        assert_eq!(stale.get::<Owner>(OWNER_KEY).await.unwrap(), Some(next));
        assert!(stale.get::<bool>("outdated").await.unwrap().is_none());
        assert_eq!(stale.expiry(), Some(Expiry::AtDateTime(expiry)));
        stale.save().await.unwrap();
        assert_eq!(
            lineage
                .authority
                .lock()
                .await
                .committed
                .as_ref()
                .unwrap()
                .expiry_date,
            expiry
        );
    }

    #[tokio::test]
    async fn deletion_and_revocation_never_upsert_aliases_or_allow_stale_creates() {
        let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
        let (original, lineage, marker) = seed(&store).await;
        let stale = independent(&store, &original);
        stale.load().await.unwrap();
        let old_id = original.id().unwrap();
        lineage.authority.lock().await.creating = true;
        original.cycle_id().await.unwrap();
        original.save().await.unwrap();
        assert_ne!(original.id(), Some(old_id));
        assert!(
            store.for_id(old_id).is_some(),
            "rotation alias permits logout"
        );
        stale.save().await.unwrap();
        assert!(store.backing.load(&old_id).await.unwrap().is_none());
        store.revoke(&lineage).await.unwrap();
        assert!(!store.current(&original).await);
        // Both a cached save and a completely independent create retain the revoked
        // ownership marker but cannot persist any usable record.
        original.save().await.unwrap();
        let late = Session::new(None, Arc::new(store.clone()), None);
        late.insert(OWNER_KEY, marker).await.unwrap();
        late.insert("authenticated", true).await.unwrap();
        late.save().await.unwrap();
        assert!(
            store
                .backing
                .load(&late.id().unwrap())
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.load(&original.id().unwrap()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn create_retains_backing_memory_store_collision_semantics() {
        let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
        let (first, lineage, _) = seed(&store).await;
        let mut collision = lineage.authority.lock().await.committed.clone().unwrap();
        let (key, next) = store.allocate(collision.expiry_date);
        let marker = Owner::fresh(key);
        collision
            .data
            .insert(OWNER_KEY.into(), serde_json::to_value(&marker).unwrap());
        {
            let mut authority = next.authority.lock().await;
            authority.permit = Some(marker);
            authority.creating = true;
        }
        store.create(&mut collision).await.unwrap();
        assert_ne!(Some(collision.id), first.id());
        assert!(store.current(&first).await);
    }

    #[tokio::test]
    async fn exact_generation_duplicates_skip_backend_failures_but_real_changes_persist() {
        for after_write in [false, true] {
            let faults = SessionFaults::default();
            let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
            let (session, lineage, _) = seed(&store).await;
            let record = lineage.authority.lock().await.committed.clone().unwrap();
            faults.fail_next(StoreOperation::Save, after_write);
            session.save().await.unwrap();
            let saved = store.backing.load(&record.id).await.unwrap().unwrap();
            assert_eq!(saved.data, record.data);
            assert_eq!(saved.expiry_date, record.expiry_date);
            session.insert("real_change", true).await.unwrap();
            session.save().await.unwrap();
            assert_eq!(
                store.backing.load(&record.id).await.unwrap().unwrap().data["real_change"],
                true
            );
            // Even a duplicate must not recreate an absent backing record.
            store.backing.delete(&record.id).await.unwrap();
            session.save().await.unwrap();
            assert!(store.backing.load(&record.id).await.unwrap().is_none());
        }
    }

    #[tokio::test]
    async fn delete_serializes_against_an_independently_cached_save() {
        let faults = SessionFaults::default();
        let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
        let (session, lineage, _) = seed(&store).await;
        let stale = independent(&store, &session);
        stale.load().await.unwrap();
        stale.insert("real_change", true).await.unwrap();
        let id = session.id().unwrap();
        let pause = faults.pause_next(StoreOperation::Delete);
        let delete = {
            let store = store.clone();
            tokio::spawn(async move { store.delete(&id).await })
        };
        pause.wait_entered().await;
        assert!(
            lineage.authority.try_lock().is_err(),
            "deletion holds the same authority lock as writes across backing I/O"
        );
        let saving = faults.pause_next(StoreOperation::Save);
        let save = tokio::spawn(async move { stale.save().await });
        saving.wait_entered().await;
        saving.release();
        assert!(
            !save.is_finished(),
            "delete holds backing serialization across I/O"
        );
        pause.release();
        delete.await.unwrap().unwrap();
        save.await.unwrap().unwrap();
        assert!(store.backing.load(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn failed_create_removes_the_direct_provisional_identifier_and_allocation() {
        for after_write in [false, true] {
            let faults = SessionFaults::default();
            let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
            let expiry = OffsetDateTime::now_utc() + Duration::minutes(5);
            let (key, lineage) = store.allocate(expiry);
            let marker = Owner::fresh(key.clone());
            {
                let mut authority = lineage.authority.lock().await;
                authority.permit = Some(marker.clone());
                authority.creating = true;
            }
            let mut proposed = tower_sessions::session::Record {
                id: tower_sessions::session::Id::default(),
                data: std::collections::HashMap::default(),
                expiry_date: expiry,
            };
            proposed
                .data
                .insert(OWNER_KEY.into(), serde_json::to_value(marker).unwrap());
            faults.fail_next(StoreOperation::Create, after_write);
            assert!(store.create(&mut proposed).await.is_err());
            assert!(store.backing.load(&proposed.id).await.unwrap().is_none());
            assert!(lineage.authority.lock().await.committed.is_none());
            let session = Session::new(None, Arc::new(store.clone()), None);
            store.rollback_binding(&session, &lineage, &key).await;
            assert!(store.lineage(&key).is_none());
            let index = store.index.lock().unwrap();
            assert!(index.lineages.is_empty());
            assert!(index.aliases.is_empty());
        }
    }

    #[tokio::test]
    async fn failed_rotated_auth_create_cleans_the_exact_provisional_id() {
        for after_write in [false, true] {
            let faults = SessionFaults::default();
            let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
            let (session, lineage, previous) = seed(&store).await;
            let old_id = session.id().unwrap();
            let marker = Owner::fresh(previous.lineage);
            {
                let mut authority = lineage.authority.lock().await;
                authority.permit = Some(marker.clone());
                authority.creating = true;
            }
            session.cycle_id().await.unwrap();
            let mut proposed = tower_sessions::session::Record {
                id: tower_sessions::session::Id::default(),
                data: std::collections::HashMap::from([
                    (OWNER_KEY.into(), serde_json::to_value(marker).unwrap()),
                    (
                        "authenticated".into(),
                        serde_json::json!({"authenticated_at": 1, "expires_at": i64::MAX}),
                    ),
                ]),
                expiry_date: OffsetDateTime::now_utc() + Duration::hours(12),
            };
            faults.fail_next(StoreOperation::Create, after_write);
            assert!(store.create(&mut proposed).await.is_err());
            store.revoke(&lineage).await.unwrap();
            for id in [old_id, proposed.id] {
                assert!(store.backing.load(&id).await.unwrap().is_none());
                assert!(store.load(&id).await.unwrap().is_none());
            }
            session.save().await.unwrap();
            assert!(
                store
                    .backing
                    .load(&session.id().unwrap())
                    .await
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[derive(Debug)]
    struct MutableClock(std::sync::Mutex<OffsetDateTime>);
    impl super::Clock for MutableClock {
        fn now(&self) -> OffsetDateTime {
            *self.0.lock().unwrap()
        }
    }

    #[tokio::test]
    async fn coordination_records_and_rotation_aliases_retire_at_absolute_deadline() {
        for ttl in [Duration::minutes(5), Duration::hours(12)] {
            assert_terminal_retirement(ttl).await;
        }
    }

    async fn assert_terminal_retirement(ttl: Duration) {
        let clock = Arc::new(MutableClock(std::sync::Mutex::new(
            OffsetDateTime::now_utc(),
        )));
        let store = AuthStore::new(clock.clone(), SessionFaults::default());
        let (session, lineage, marker) = seed(&store).await;
        let old_id = session.id().unwrap();
        let expiry = clock.now() + ttl;
        lineage.authority.lock().await.creating = true;
        session.cycle_id().await.unwrap();
        session.set_expiry(Some(Expiry::AtDateTime(expiry)));
        session.save().await.unwrap();
        let rotated_id = session.id().unwrap();
        let expiry = lineage
            .authority
            .lock()
            .await
            .committed
            .as_ref()
            .unwrap()
            .expiry_date;
        {
            let index = store.index.lock().unwrap();
            assert_eq!(index.lineages.len(), 1);
            assert_eq!(index.aliases.len(), 2);
        }
        store.revoke(&lineage).await.unwrap();
        assert!(
            store.for_id(old_id).is_some(),
            "revocation keeps the alias deadline"
        );
        assert!(store.for_id(rotated_id).is_some());
        assert!(store.backing.load(&old_id).await.unwrap().is_none());
        assert!(store.backing.load(&rotated_id).await.unwrap().is_none());
        *clock.0.lock().unwrap() = expiry - Duration::nanoseconds(1);
        assert!(store.lineage(&marker.lineage).is_some());
        assert!(store.for_id(old_id).is_some());
        *clock.0.lock().unwrap() = expiry;
        assert!(store.lineage(&marker.lineage).is_none());
        assert!(store.for_id(session.id().unwrap()).is_none());
        {
            let index = store.index.lock().unwrap();
            assert!(index.aliases.is_empty());
            assert!(index.lineages.is_empty());
        }
        // Pruning terminal coordination must never re-authorize an old marker.
        session.save().await.unwrap();
        assert!(store.backing.load(&rotated_id).await.unwrap().is_none());
        let stale_create = Session::new(None, Arc::new(store.clone()), None);
        stale_create.insert(OWNER_KEY, marker).await.unwrap();
        stale_create.save().await.unwrap();
        assert!(
            store
                .backing
                .load(&stale_create.id().unwrap())
                .await
                .unwrap()
                .is_none()
        );
        let index = store.index.lock().unwrap();
        assert!(index.lineages.is_empty());
        assert!(index.aliases.is_empty());
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
