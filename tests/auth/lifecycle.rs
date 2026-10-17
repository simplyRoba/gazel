use std::future::Future;

use axum::http::HeaderValue;
use gazel::auth::{SessionFaults, StoreOperation};
use tokio::sync::Semaphore;

use super::*;

#[derive(Debug)]
pub(super) struct TokenGate {
    entered: Semaphore,
    released: Semaphore,
}

impl TokenGate {
    async fn install(provider: &MockProvider) -> Arc<Self> {
        let gate = Arc::new(Self {
            entered: Semaphore::new(0),
            released: Semaphore::new(0),
        });
        *provider.state.token_gate.lock().await = Some(Arc::clone(&gate));
        gate
    }

    async fn entered(&self) {
        self.entered.acquire().await.unwrap().forget();
    }
    fn release(&self) {
        self.released.add_permits(1);
    }
}

pub(super) async fn wait_token_gate(state: &ProviderState) {
    let gate = state.token_gate.lock().await.take();
    if let Some(gate) = gate {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
    }
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(std::time::Duration::from_secs(5), future)
        .await
        .expect("unrelated lifecycle work must not stall")
}

async fn request(app: &Router, method: &str, path: &str, cookie: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn replacement(app: &Router, provider: &MockProvider, cookie: &str) -> LoginAttempt {
    let response = request(app, "GET", "/auth/login?return_to=%2Fnew", cookie).await;
    let url = url::Url::parse(&response_location(&response)).unwrap();
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let attempt = LoginAttempt {
        cookie: response_cookie(&response),
        state: query["state"].clone(),
        nonce: query["nonce"].clone(),
        challenge: query["code_challenge"].clone(),
    };
    provider
        .expect_login(
            attempt.state.clone(),
            attempt.nonce.clone(),
            attempt.challenge.clone(),
        )
        .await;
    attempt
}

async fn with_faults() -> (MockProvider, Router, SessionFaults) {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let faults = SessionFaults::default();
    let auth = Authentication::bootstrap_with_session_faults(
        provider.config(),
        Arc::new(TestClock::new(OffsetDateTime::now_utc())),
        faults.clone(),
    )
    .await
    .unwrap();
    (provider, server_app(Some(auth)).await, faults)
}

async fn assert_unauthenticated(app: &Router, cookie: &str) {
    assert_eq!(
        request(app, "GET", "/api/info", cookie).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

async fn preloaded_request(
    State(gate): State<Arc<TokenGate>>,
    session: tower_sessions::Session,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let phase = request
        .headers()
        .get("x-pause")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if phase.is_some() {
        // Every HTTP request has its own Session::new cache, not a cloned Session.
        session.get::<String>("oidc_transaction").await.unwrap();
    }
    if phase.as_deref() == Some("before") {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
    }
    let mut response = next.run(request).await;
    if phase.as_deref() == Some("after") || phase.as_deref() == Some("empty") {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
        if phase.as_deref() == Some("empty") {
            session.clear().await;
        }
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("unrelated=kept; Path=/"),
        );
    }
    response
}

async fn preloaded_app(provider: &MockProvider, faults: SessionFaults) -> (Router, Arc<TokenGate>) {
    preloaded_app_with_clock(
        provider,
        faults,
        Arc::new(TestClock::new(OffsetDateTime::now_utc())),
    )
    .await
}

async fn preloaded_app_with_clock(
    provider: &MockProvider,
    faults: SessionFaults,
    clock: Arc<TestClock>,
) -> (Router, Arc<TokenGate>) {
    let auth = Arc::new(
        Authentication::bootstrap_with_session_faults(provider.config(), clock, faults)
            .await
            .unwrap(),
    );
    let gate = Arc::new(TokenGate {
        entered: Semaphore::new(0),
        released: Semaphore::new(0),
    });
    let app = auth
        .protocol_routes()
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&gate),
            preloaded_request,
        ))
        .layer(auth.session_layer());
    (app, gate)
}

fn paused_login(app: Router, cookie: String, phase: &str) -> tokio::task::JoinHandle<Response> {
    let phase = phase.to_owned();
    tokio::spawn(async move {
        app.oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fdelayed")
                .header(header::COOKIE, cookie)
                .header("x-pause", phase)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    })
}

async fn attempt_from_response(
    provider: &MockProvider,
    response: &Response,
    cookie: String,
) -> LoginAttempt {
    let url = url::Url::parse(&response_location(response)).unwrap();
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let attempt = LoginAttempt {
        cookie,
        state: query["state"].clone(),
        nonce: query["nonce"].clone(),
        challenge: query["code_challenge"].clone(),
    };
    provider
        .expect_login(
            attempt.state.clone(),
            attempt.nonce.clone(),
            attempt.challenge.clone(),
        )
        .await;
    attempt
}

#[tokio::test]
async fn independently_preloaded_login_cannot_replace_the_newer_winning_binding() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let (app, gate) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie.clone(), "before");
    bounded(gate.entered()).await;
    let middle = bounded(replacement(&app, &provider, &original.cookie)).await;
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(stale.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(
        response_location(&callback_request(&app, &original, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(
        response_location(&callback_request(&app, &middle, "").await),
        "/new"
    );
    assert_eq!(
        response_location(&callback_request(&app, &middle, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn logout_rejects_preloaded_login_but_allows_a_new_cookieless_lineage() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let (app, gate) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie.clone(), "before");
    bounded(gate.entered()).await;
    let logout = bounded(request(&app, "POST", "/auth/logout", &original.cookie)).await;
    assert_eq!(response_location(&logout), "/login?logged_out=1");
    let fresh = bounded(login_attempt(&app, &provider, "%2Ffresh")).await;
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(
        response_location(&callback_request(&app, &original, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.token_count(), 0);
    assert_eq!(
        response_location(&callback_request(&app, &fresh, "").await),
        "/fresh"
    );
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn pruned_preloaded_owner_cannot_allocate_a_fresh_lineage() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let clock = Arc::new(TestClock::new(OffsetDateTime::now_utc()));
    let (app, gate) =
        preloaded_app_with_clock(&provider, SessionFaults::default(), clock.clone()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie, "before");
    bounded(gate.entered()).await;
    clock.advance(Duration::minutes(5));
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(provider.token_count(), 0);
    let unknown = request(&app, "GET", "/auth/login", "gazel_session=tampered").await;
    assert!(response_location(&unknown).starts_with(provider.issuer().trim_end_matches('/')));
    assert!(response_set_cookie(&unknown).starts_with("gazel_session="));
}

#[tokio::test]
async fn rotation_alias_cannot_rebind_the_authenticated_or_revoked_lineage() {
    let (provider, app, _) = with_faults().await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let authenticated = callback_request(&app, &original, "").await;
    let cookie = response_cookie(&authenticated);
    for revoked in [false, true] {
        if revoked {
            request(&app, "POST", "/auth/logout", &cookie).await;
        }
        let stale = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &original.cookie,
        )
        .await;
        assert_eq!(
            response_location(&stale),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert!(
            stale
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .all(|c| c.to_str().unwrap().starts_with("gazel_session=;"))
        );
        if !revoked {
            assert_eq!(
                request(&app, "GET", "/api/info", &cookie).await.status(),
                StatusCode::OK
            );
        }
    }
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn preloading_a_missing_rotation_alias_does_not_erase_its_request_identity() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let (app, _) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let authenticated = callback_request(&app, &original, "").await;
    let cookie = response_cookie(&authenticated);
    for (method, path) in [
        ("GET", "/auth/login?return_to=%2Fstale"),
        ("POST", "/auth/logout"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::COOKIE, &original.cookie)
                    .header("x-pause", "load")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if method == "GET" {
            assert_eq!(
                response_location(&response),
                "/login?error=provider_unavailable&return_to=%2Fstale"
            );
            assert!(
                response.headers().get(header::SET_COOKIE).is_none(),
                "an alias response cannot expire the winner's cookie"
            );
            assert_eq!(
                response_location(
                    &request(&app, "GET", "/auth/login?return_to=%2Fstill", &cookie).await
                ),
                "/still"
            );
        } else {
            assert_eq!(response_location(&response), "/login?logged_out=1");
            assert!(response_set_cookie(&response).contains("Max-Age=0"));
            assert_eq!(
                response_location(
                    &request(&app, "GET", "/auth/login?return_to=%2Fstill", &cookie).await
                ),
                "/login?error=provider_unavailable&return_to=%2Fstill"
            );
        }
    }
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn callback_expiry_during_completion_reload_is_fail_closed_not_a_panic() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let clock = Arc::new(TestClock::new(OffsetDateTime::now_utc()));
    let faults = SessionFaults::default();
    let auth = Authentication::bootstrap_with_session_faults(
        provider.config(),
        clock.clone(),
        faults.clone(),
    )
    .await
    .unwrap();
    let app = server_app(Some(auth)).await;
    let attempt = login_attempt(&app, &provider, "%2Fstored").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    let reload = faults.pause_next(StoreOperation::Load);
    gate.release();
    bounded(reload.wait_entered()).await;
    clock.advance(Duration::minutes(5));
    reload.release();
    let failed = bounded(task).await.unwrap();
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=authentication_failed&return_to=%2Fstored"
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn callback_completion_reload_error_uses_provider_unavailable_and_stored_target() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fstored").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    faults.fail_next(StoreOperation::Load, false);
    gate.release();
    let failed = bounded(task).await.unwrap();
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fstored"
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn late_failed_auto_save_cannot_restore_old_binding_or_overwrite_cookie() {
    for phase in ["after", "empty"] {
        let provider = MockProvider::start(ProviderOptions::default()).await;
        let faults = SessionFaults::default();
        let (app, gate) = preloaded_app(&provider, faults.clone()).await;
        let original = login_attempt(&app, &provider, "%2Foriginal").await;
        let task = paused_login(app.clone(), original.cookie.clone(), phase);
        bounded(gate.entered()).await;
        let winner = bounded(replacement(&app, &provider, &original.cookie)).await;
        // The older handler already saved explicitly. Its later middleware save
        // must not retry/roll back or turn this intentional 303 into a 500.
        if phase == "after" {
            faults.fail_next(StoreOperation::Save, true);
        }
        gate.release();
        let late = bounded(task).await.unwrap();
        assert_eq!(late.status(), StatusCode::SEE_OTHER);
        let cookies: Vec<_> = late.headers().get_all(header::SET_COOKIE).iter().collect();
        assert_eq!(
            cookies,
            vec![&HeaderValue::from_static("unrelated=kept; Path=/")]
        );
        let obsolete = attempt_from_response(&provider, &late, original.cookie).await;
        assert_eq!(
            response_location(&callback_request(&app, &obsolete, "").await),
            "/login?error=authentication_failed&return_to=%2F"
        );
        assert_eq!(
            response_location(&callback_request(&app, &winner, "").await),
            "/new"
        );
        assert_eq!(provider.token_count(), 1);
    }
}

#[tokio::test]
async fn failed_replacement_preserves_an_already_exchanging_completion_owner() {
    let (provider, app, faults) = with_faults().await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let original = original.clone();
        tokio::spawn(async move { callback_request(&app, &original, "").await })
    };
    bounded(gate.entered()).await;
    faults.fail_next(StoreOperation::Save, true);
    let failed = bounded(request(
        &app,
        "GET",
        "/auth/login?return_to=%2Fnew",
        &original.cookie,
    ))
    .await;
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fnew"
    );
    gate.release();
    let current = bounded(task).await.unwrap();
    assert_eq!(response_location(&current), "/original");
    assert_eq!(
        request(&app, "GET", "/api/info", &response_cookie(&current))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn replacement_revokes_blocked_completion_without_stalling_other_browsers() {
    let (provider, app, _) = with_faults().await;
    let old = login_attempt(&app, &provider, "%2Fold").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let old = old.clone();
        tokio::spawn(async move { callback_request(&app, &old, "").await })
    };
    bounded(gate.entered()).await;
    let new = bounded(replacement(&app, &provider, &old.cookie)).await;
    let other = bounded(login_attempt(&app, &provider, "%2Fother")).await;
    let authenticated = bounded(callback_request(&app, &other, "")).await;
    let other_cookie = response_cookie(&authenticated);
    assert_eq!(
        bounded(request(&app, "GET", "/api/info", &other_cookie))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        bounded(request(&app, "POST", "/auth/logout", &other_cookie))
            .await
            .status(),
        StatusCode::SEE_OTHER
    );
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=authentication_failed&return_to=%2Fold"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    let current = callback_request(&app, &new, "").await;
    assert_eq!(response_location(&current), "/new");
    assert_eq!(provider.token_count(), 3);
    assert_unauthenticated(&app, &old.cookie).await;
}

#[tokio::test]
async fn logout_revokes_blocked_exchange_and_preserves_anonymous_cookie_expiry() {
    let (provider, app, _) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    for cookie in [&attempt.cookie, ""] {
        let logout = bounded(request(&app, "POST", "/auth/logout", cookie)).await;
        assert_eq!(response_location(&logout), "/login?logged_out=1");
        assert!(response_set_cookie(&logout).contains("Max-Age=0"));
    }
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=authentication_failed&return_to=%2Fsettings"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn logout_with_rotation_alias_suppresses_late_middleware_save_and_cookie() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    // Callback explicitly creates the rotated record; its next save is the
    // outer SessionManager's automatic duplicate, after the lifecycle gate exits.
    let pause = faults.pause_next(StoreOperation::Save);
    faults.fail_next(StoreOperation::Save, true);
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(pause.wait_entered()).await;
    let logout = bounded(request(&app, "POST", "/auth/logout", &attempt.cookie)).await;
    assert_eq!(response_location(&logout), "/login?logged_out=1");
    pause.release();
    let late = bounded(task).await.unwrap();
    assert_eq!(
        late.status(),
        StatusCode::SEE_OTHER,
        "obsolete save is suppression, not middleware 500"
    );
    assert!(late.headers().get(header::SET_COOKIE).is_none());
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(
        response_location(&callback_request(&app, &attempt, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
}

#[tokio::test]
async fn failed_replacement_restores_prior_binding_and_exchange_permission() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        let prior = login_attempt(&app, &provider, "%2Fprior").await;
        faults.fail_next(StoreOperation::Save, after_write);
        let failed = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &prior.cookie,
        )
        .await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert_eq!(
            response_location(&callback_request(&app, &prior, "").await),
            "/prior"
        );
        assert_eq!(provider.token_count(), 1);
    }
}

#[tokio::test]
async fn failed_authenticated_create_and_rotation_cannot_be_retried_by_middleware() {
    for operation in [StoreOperation::Create, StoreOperation::Delete] {
        for after_write in [false, true] {
            let (provider, app, faults) = with_faults().await;
            let attempt = login_attempt(&app, &provider, "%2Fstored").await;
            faults.fail_next(operation, after_write);
            let failed = callback_request(&app, &attempt, "").await;
            assert_eq!(
                response_location(&failed),
                "/login?error=provider_unavailable&return_to=%2Fstored"
            );
            assert_unauthenticated(&app, &attempt.cookie).await;
            if let Some(cookie) = failed.headers().get(header::SET_COOKIE) {
                let cookie = cookie.to_str().unwrap().split(';').next().unwrap();
                assert_unauthenticated(&app, cookie).await;
                assert!(
                    cookie.ends_with('='),
                    "no provisional auth cookie may escape"
                );
            }
            assert_eq!(provider.token_count(), 1);
        }
    }
}

#[tokio::test]
async fn initial_binding_and_load_failures_do_not_leave_usable_orphans() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        faults.fail_next(StoreOperation::Create, after_write);
        let failed = request(&app, "GET", "/auth/login?return_to=%2Frequested", "").await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert!(failed.headers().get(header::SET_COOKIE).is_none());
        let attempt = login_attempt(&app, &provider, "%2Fprior").await;
        faults.fail_next(StoreOperation::Load, false);
        let failed = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &attempt.cookie,
        )
        .await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert_eq!(
            response_location(&callback_request(&app, &attempt, "").await),
            "/prior"
        );
    }
}

#[tokio::test]
async fn logout_delete_failures_return_500_and_expire_cookie_without_resurrection() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
        let authenticated = callback_request(&app, &attempt, "").await;
        let cookie = response_cookie(&authenticated);
        faults.fail_next(StoreOperation::Delete, after_write);
        let failed = request(&app, "POST", "/auth/logout", &cookie).await;
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(failed.headers().get(header::LOCATION).is_none());
        assert!(response_set_cookie(&failed).contains("Max-Age=0"));
        assert_eq!(failed.headers()[header::CACHE_CONTROL], "no-store");
        assert_unauthenticated(&app, &cookie).await;
        assert_unauthenticated(&app, &attempt.cookie).await;
    }
}

#[derive(Clone, Copy)]
enum PostHandlerFailureMode {
    SaveBeforeWrite,
    SaveAfterWrite,
    Callback500PreserveExtensions,
    Callback500ReplaceResponse,
}

#[derive(Clone)]
struct PostHandlerFailure {
    faults: SessionFaults,
    mode: PostHandlerFailureMode,
    provisional: Arc<StdMutex<Option<tower_sessions::Session>>>,
}

async fn mutate_after_callback(
    State(failure): State<PostHandlerFailure>,
    session: tower_sessions::Session,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let callback = request.uri().path() == "/auth/callback";
    let mut response = next.run(request).await;
    if callback
        && session
            .get::<serde_json::Value>("authenticated")
            .await
            .unwrap()
            .is_some()
    {
        *failure.provisional.lock().unwrap() = Some(session.clone());
        match failure.mode {
            PostHandlerFailureMode::SaveBeforeWrite | PostHandlerFailureMode::SaveAfterWrite => {
                // Force a backing write rather than an idempotent duplicate save.
                session.insert("test_post_handler", true).await.unwrap();
                failure.faults.fail_next(
                    StoreOperation::Save,
                    matches!(failure.mode, PostHandlerFailureMode::SaveAfterWrite),
                );
            }
            PostHandlerFailureMode::Callback500PreserveExtensions => {
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response.headers_mut().remove(header::LOCATION);
            }
            PostHandlerFailureMode::Callback500ReplaceResponse => {
                // Drop CallbackIntent and all other handler response extensions.
                response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                assert!(response.extensions().is_empty());
            }
        }
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("unrelated=kept; Path=/"),
        );
    }
    response
}

#[tokio::test]
async fn middleware_failure_preserves_callback_target_and_revokes_provisional_id() {
    for mode in [
        PostHandlerFailureMode::SaveBeforeWrite,
        PostHandlerFailureMode::SaveAfterWrite,
    ] {
        bounded(assert_middleware_failure(mode)).await;
    }
}

#[tokio::test]
async fn callback_500_with_preserved_extensions_revokes_provisional_id() {
    bounded(assert_middleware_failure(
        PostHandlerFailureMode::Callback500PreserveExtensions,
    ))
    .await;
}

#[tokio::test]
async fn callback_500_replacing_all_extensions_preserves_target_and_revokes_provisional_id() {
    bounded(assert_middleware_failure(
        PostHandlerFailureMode::Callback500ReplaceResponse,
    ))
    .await;
}

async fn assert_middleware_failure(mode: PostHandlerFailureMode) {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let failure = PostHandlerFailure {
        faults: SessionFaults::default(),
        mode,
        provisional: Arc::default(),
    };
    let auth = Arc::new(
        Authentication::bootstrap_with_session_faults(
            provider.config(),
            Arc::new(TestClock::new(OffsetDateTime::now_utc())),
            failure.faults.clone(),
        )
        .await
        .unwrap(),
    );
    let protected = server_app(Some((*auth).clone())).await;
    let app = auth
        .protocol_routes()
        .layer(axum::middleware::from_fn_with_state(
            failure.clone(),
            mutate_after_callback,
        ))
        .layer(auth.session_layer());
    let attempt = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata%23export").await;
    let failed = callback_request(&app, &attempt, "").await;
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fsettings%3Ftab%3Ddata%23export"
    );
    let cookies: Vec<_> = failed
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|c| c.to_str().unwrap())
        .collect();
    assert!(cookies.contains(&"unrelated=kept; Path=/"));
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("gazel_session=;") && c.contains("Max-Age=0"))
    );
    assert!(
        cookies
            .iter()
            .filter(|c| c.starts_with("gazel_session="))
            .all(|c| { c.starts_with("gazel_session=;") && c.contains("Max-Age=0") }),
        "no provisional auth cookie may escape"
    );
    assert_unauthenticated(&protected, &attempt.cookie).await;
    let provisional = failure.provisional.lock().unwrap().take().unwrap();
    let id = provisional.id().unwrap();
    provisional.load().await.unwrap();
    assert!(
        provisional
            .get::<serde_json::Value>("authenticated")
            .await
            .unwrap()
            .is_none()
    );
    assert_ne!(
        provisional.id(),
        Some(id),
        "the proposed identifier has no loadable record"
    );
    assert_eq!(provider.token_count(), 1);
}

async fn replace_logout_response(request: Request<Body>, next: axum::middleware::Next) -> Response {
    let logout = request.uri().path() == "/auth/logout";
    let response = next.run(request).await;
    if !logout {
        return response;
    }
    // Discard both LogoutExpiry and the handler's explicit removal cookie.
    let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
    assert!(response.extensions().is_empty());
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_static("unrelated=kept; Path=/"),
    );
    response
}

#[tokio::test]
async fn logout_500_response_replacement_still_expires_cookie_and_revokes_authentication() {
    bounded(async {
        let provider = MockProvider::start(ProviderOptions::default()).await;
        let auth = Arc::new(
            Authentication::bootstrap_with_session_faults(
                provider.config(),
                Arc::new(TestClock::new(OffsetDateTime::now_utc())),
                SessionFaults::default(),
            )
            .await
            .unwrap(),
        );
        let protected = server_app(Some((*auth).clone())).await;
        let app = auth
            .protocol_routes()
            .layer(axum::middleware::from_fn(replace_logout_response))
            .layer(auth.session_layer());
        let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
        let authenticated = callback_request(&app, &attempt, "").await;
        assert_eq!(authenticated.status(), StatusCode::SEE_OTHER);
        let cookie = response_cookie(&authenticated);
        assert_eq!(
            request(&protected, "GET", "/api/info", &cookie)
                .await
                .status(),
            StatusCode::OK
        );

        let failed = request(&app, "POST", "/auth/logout", &cookie).await;
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(failed.headers().get(header::LOCATION).is_none());
        let cookies: Vec<_> = failed
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|c| c.to_str().unwrap())
            .collect();
        assert!(cookies.contains(&"unrelated=kept; Path=/"));
        assert!(
            cookies
                .iter()
                .any(|c| { c.starts_with("gazel_session=;") && c.contains("Max-Age=0") })
        );
        assert_unauthenticated(&protected, &cookie).await;
        assert_unauthenticated(&protected, &attempt.cookie).await;
        assert_eq!(provider.token_count(), 1);
    })
    .await;
}

#[tokio::test]
async fn middleware_duplicate_save_is_not_a_second_backing_write() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    faults.fail_next(StoreOperation::Save, true);
    let success = callback_request(&app, &attempt, "").await;
    assert_eq!(success.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&success), "/settings");
    let cookie = response_cookie(&success);
    assert_eq!(
        request(&app, "GET", "/api/info", &cookie).await.status(),
        StatusCode::OK
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
}
