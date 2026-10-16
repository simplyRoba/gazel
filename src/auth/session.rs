use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{Request, header};
use axum::response::Response;
use tower::{Layer, Service};
use tower_sessions::cookie::{Key, SameSite};
use tower_sessions::service::PrivateCookie;
use tower_sessions::{Session, SessionManagerLayer};

use super::Clock;
use super::session_store::{AuthStore, SessionFaults};

const COOKIE_KEY_LENGTH: usize = 64;
pub const SESSION_COOKIE_NAME: &str = "gazel_session";

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
            target: super::flow::validate_return_to(Some(target)),
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
            *response = super::flow::failure_redirect(
                super::flow::FailureCode::ProviderUnavailable,
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

    async fn owned_session(
        backend: &SessionBackend,
    ) -> (Session, Arc<super::super::session_store::Lineage>) {
        use super::super::session_store::{OWNER_KEY, Owner};
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
        use super::super::session_store::{OWNER_KEY, Owner};

        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::flow::SystemClock),
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
            .header(header::SET_COOKIE, "gazel_session=obsolete; Path=/")
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
        use super::super::session_store::{OWNER_KEY, Owner};
        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::flow::SystemClock),
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
        use super::super::session_store::{OWNER_KEY, Owner};
        let backend = SessionBackend::new(
            false,
            Arc::new(super::super::flow::SystemClock),
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
            Arc::new(super::super::flow::SystemClock),
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
