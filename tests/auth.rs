use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use http_body_util::BodyExt;
use openidconnect::core::{
    CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey,
};
use openidconnect::{
    AccessToken, Audience, EmptyAdditionalClaims, IssuerUrl, JsonWebKeyId, Nonce,
    PrivateSigningKey, StandardClaims, SubjectIdentifier,
};
use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};
use tokio::net::TcpListener;
use tokio::sync::{Barrier, Mutex};
use tower::ServiceExt;

use gazel::auth::{Authentication, Clock};
use gazel::config::OidcConfig;

const RETURN_TO_MAX_BYTES: usize = 2_048;
const CLIENT_ID: &str = "gazel-client";
const CLIENT_SECRET: &str = "gazel-secret";
const RSA_PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
MIIEowIBAAKCAQEAn4EPtAOCc9AlkeQHPzHStgAbgs7bTZLwUBZdR8/KuKPEHLd4\n\
rHVTeT+O+XV2jRojdNhxJWTDvNd7nqQ0VEiZQHz/AJmSCpMaJMRBSFKrKb2wqVwG\n\
U/NsYOYL+QtiWN2lbzcEe6XC0dApr5ydQLrHqkHHig3RBordaZ6Aj+oBHqFEHYpP\n\
e7Tpe+OfVfHd1E6cS6M1FZcD1NNLYD5lFHpPI9bTwJlsde3uhGqC0ZCuEHg8lhzw\n\
OHrtIQbS0FVbb9k3+tVTU4fg/3L/vniUFAKwuCLqKnS2BYwdq/mzSnbLY7h/qixo\n\
R7jig3//kRhuaxwUkRz5iaiQkqgc5gHdrNP5zwIDAQABAoIBAG1lAvQfhBUSKPJK\n\
Rn4dGbshj7zDSr2FjbQf4pIh/ZNtHk/jtavyO/HomZKV8V0NFExLNi7DUUvvLiW7\n\
0PgNYq5MDEjJCtSd10xoHa4QpLvYEZXWO7DQPwCmRofkOutf+NqyDS0QnvFvp2d+\n\
Lov6jn5C5yvUFgw6qWiLAPmzMFlkgxbtjFAWMJB0zBMy2BqjntOJ6KnqtYRMQUxw\n\
TgXZDF4rhYVKtQVOpfg6hIlsaoPNrF7dofizJ099OOgDmCaEYqM++bUlEHxgrIVk\n\
wZz+bg43dfJCocr9O5YX0iXaz3TOT5cpdtYbBX+C/5hwrqBWru4HbD3xz8cY1TnD\n\
qQa0M8ECgYEA3Slxg/DwTXJcb6095RoXygQCAZ5RnAvZlno1yhHtnUex/fp7AZ/9\n\
nRaO7HX/+SFfGQeutao2TDjDAWU4Vupk8rw9JR0AzZ0N2fvuIAmr/WCsmGpeNqQn\n\
ev1T7IyEsnh8UMt+n5CafhkikzhEsrmndH6LxOrvRJlsPp6Zv8bUq0kCgYEAuKE2\n\
dh+cTf6ERF4k4e/jy78GfPYUIaUyoSSJuBzp3Cubk3OCqs6grT8bR/cu0Dm1MZwW\n\
mtdqDyI95HrUeq3MP15vMMON8lHTeZu2lmKvwqW7anV5UzhM1iZ7z4yMkuUwFWoB\n\
vyY898EXvRD+hdqRxHlSqAZ192zB3pVFJ0s7pFcCgYAHw9W9eS8muPYv4ZhDu/fL\n\
2vorDmD1JqFcHCxZTOnX1NWWAj5hXzmrU0hvWvFC0P4ixddHf5Nqd6+5E9G3k4E5\n\
2IwZCnylu3bqCWNh8pT8T3Gf5FQsfPT5530T2BcsoPhUaeCnP499D+rb2mTnFYeg\n\
mnTT1B/Ue8KGLFFfn16GKQKBgAiw5gxnbocpXPaO6/OKxFFZ+6c0OjxfN2PogWce\n\
TU/k6ZzmShdaRKwDFXisxRJeNQ5Rx6qgS0jNFtbDhW8E8WFmQ5urCOqIOYk28EBi\n\
At4JySm4v+5P7yYBh8B8YD2l9j57z/s8hJAxEbn/q8uHP2ddQqvQKgtsni+pHSk9\n\
XGBfAoGBANz4qr10DdM8DHhPrAb2YItvPVz/VwkBd1Vqj8zCpyIEKe/07oKOvjWQ\n\
SgkLDH9x2hBgY01SbP43CvPk0V72invu2TGkI/FXwXWJLLG7tDSgw4YyfhrYrHmg\n\
1Vre3XB9HH8MYBVB6UIexaAq4xSeoemRKTBesZro7OKjKT8/GmiO\n\
-----END RSA PRIVATE KEY-----";

#[derive(Clone, Debug)]
enum MetadataMode {
    Valid,
    Malformed,
    Redirect,
}

#[derive(Clone, Debug)]
enum JwksMode {
    Key(String),
    Empty,
    Unusable,
    ServerUnavailable,
    Redirect,
}

#[derive(Clone, Debug)]
enum TokenMode {
    Valid,
    Malformed,
    MalformedIdToken,
    ProviderError,
    ServerUnavailable,
    MissingNonce,
    WrongNonce,
    WrongIssuer,
    WrongAudience,
    Expired,
    InvalidAccessTokenHash,
    AdditionalAudience,
    MissingIdToken,
    CorruptSignature,
    Redirect,
}

#[derive(Clone, Debug)]
struct ProviderOptions {
    metadata_mode: MetadataMode,
    auth_methods: Option<Vec<String>>,
    issuer_override: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    jwks_endpoint: Option<String>,
    include_authorization_endpoint: bool,
    include_token_endpoint: bool,
    include_jwks_endpoint: bool,
    response_types: Vec<String>,
    grant_types: Option<Vec<String>>,
    signing_algorithms: Vec<String>,
    jwks_mode: JwksMode,
    token_mode: TokenMode,
    signing_key_id: String,
    expected_nonce: Option<String>,
    expected_challenge: Option<String>,
}

impl Default for ProviderOptions {
    fn default() -> Self {
        Self {
            metadata_mode: MetadataMode::Valid,
            auth_methods: Some(vec![String::from("client_secret_basic")]),
            issuer_override: None,
            authorization_endpoint: None,
            token_endpoint: None,
            jwks_endpoint: None,
            include_authorization_endpoint: true,
            include_token_endpoint: true,
            include_jwks_endpoint: true,
            response_types: vec![String::from("code")],
            grant_types: Some(vec![String::from("authorization_code")]),
            signing_algorithms: vec![String::from("RS256")],
            jwks_mode: JwksMode::Key(String::from("key-1")),
            token_mode: TokenMode::Valid,
            signing_key_id: String::from("key-1"),
            expected_nonce: None,
            expected_challenge: None,
        }
    }
}

#[derive(Debug, Default)]
struct ProviderCounters {
    discovery: AtomicUsize,
    jwks: AtomicUsize,
    token: AtomicUsize,
    redirect_target: AtomicUsize,
}

#[derive(Clone, Debug)]
struct TokenRequest {
    authorization: Option<String>,
    form: HashMap<String, String>,
}

#[derive(Debug)]
struct ProviderState {
    issuer: String,
    options: Mutex<ProviderOptions>,
    counters: ProviderCounters,
    token_requests: Mutex<Vec<TokenRequest>>,
    issued_id_tokens: Mutex<Vec<String>>,
    expected_logins: Mutex<HashMap<String, (String, String)>>,
    token_barrier: Mutex<Option<Arc<Barrier>>>,
}

struct MockProvider {
    state: Arc<ProviderState>,
    task: tokio::task::JoinHandle<()>,
}

impl MockProvider {
    async fn start(options: ProviderOptions) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback provider should bind");
        let issuer = format!(
            "http://{}/",
            listener.local_addr().expect("address should exist")
        );
        let state = Arc::new(ProviderState {
            issuer,
            options: Mutex::new(options),
            counters: ProviderCounters::default(),
            token_requests: Mutex::new(Vec::new()),
            issued_id_tokens: Mutex::new(Vec::new()),
            expected_logins: Mutex::new(HashMap::new()),
            token_barrier: Mutex::new(None),
        });
        let router = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/authorize", get(authorize))
            .route("/token", post(token))
            .route("/jwks", get(jwks))
            .route("/redirect-target", get(redirect_target))
            .with_state(Arc::clone(&state));
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("mock provider should serve");
        });
        Self { state, task }
    }

    fn issuer(&self) -> &str {
        &self.state.issuer
    }

    fn config(&self) -> OidcConfig {
        OidcConfig {
            external_url: url::Url::parse("https://gazel.example/").expect("URL should parse"),
            callback_url: url::Url::parse("https://gazel.example/auth/callback")
                .expect("URL should parse"),
            issuer: self.issuer().to_owned(),
            client_id: String::from(CLIENT_ID),
            client_secret: String::from(CLIENT_SECRET),
            provider_name: String::from("Test Provider"),
        }
    }

    async fn set_options(&self, update: impl FnOnce(&mut ProviderOptions)) {
        let mut options = self.state.options.lock().await;
        update(&mut options);
    }

    async fn set_barrier(&self, parties: usize) {
        *self.state.token_barrier.lock().await = Some(Arc::new(Barrier::new(parties)));
    }

    async fn expect_login(&self, code: String, nonce: String, challenge: String) {
        self.state
            .expected_logins
            .lock()
            .await
            .insert(code, (nonce, challenge));
    }

    async fn token_requests(&self) -> Vec<TokenRequest> {
        self.state.token_requests.lock().await.clone()
    }

    async fn issued_id_tokens(&self) -> Vec<String> {
        self.state.issued_id_tokens.lock().await.clone()
    }

    fn discovery_count(&self) -> usize {
        self.state.counters.discovery.load(Ordering::SeqCst)
    }

    fn jwks_count(&self) -> usize {
        self.state.counters.jwks.load(Ordering::SeqCst)
    }

    fn token_count(&self) -> usize {
        self.state.counters.token.load(Ordering::SeqCst)
    }

    fn redirect_count(&self) -> usize {
        self.state.counters.redirect_target.load(Ordering::SeqCst)
    }
}

impl Drop for MockProvider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Debug)]
struct TestClock {
    now: StdMutex<OffsetDateTime>,
    monotonic_now: StdMutex<std::time::Instant>,
}

impl TestClock {
    fn new(now: OffsetDateTime) -> Self {
        Self {
            now: StdMutex::new(now),
            monotonic_now: StdMutex::new(std::time::Instant::now()),
        }
    }

    fn advance(&self, duration: Duration) {
        self.advance_utc(duration);
        self.advance_monotonic(duration);
    }

    fn advance_utc(&self, duration: Duration) {
        let mut now = self.now.lock().expect("test clock should not be poisoned");
        *now += duration;
    }

    fn advance_monotonic(&self, duration: Duration) {
        let elapsed = std::time::Duration::try_from(duration)
            .expect("test clock should only advance forward");
        let mut monotonic_now = self
            .monotonic_now
            .lock()
            .expect("test clock should not be poisoned");
        *monotonic_now += elapsed;
    }
}

impl Clock for TestClock {
    fn now(&self) -> OffsetDateTime {
        *self.now.lock().expect("test clock should not be poisoned")
    }

    fn monotonic_now(&self) -> std::time::Instant {
        *self
            .monotonic_now
            .lock()
            .expect("test clock should not be poisoned")
    }
}

async fn discovery(State(state): State<Arc<ProviderState>>) -> Response {
    state.counters.discovery.fetch_add(1, Ordering::SeqCst);
    let options = state.options.lock().await.clone();
    match options.metadata_mode {
        MetadataMode::Malformed => {
            ([(header::CONTENT_TYPE, "application/json")], "{not-json").into_response()
        }
        MetadataMode::Redirect => (
            StatusCode::FOUND,
            [(header::LOCATION, format!("{}redirect-target", state.issuer))],
        )
            .into_response(),
        MetadataMode::Valid => {
            let issuer = options
                .issuer_override
                .unwrap_or_else(|| state.issuer.clone());
            let mut value = json!({
                "issuer": issuer,
                "response_types_supported": options.response_types,
                "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": options.signing_algorithms,
            });
            if options.include_authorization_endpoint {
                value["authorization_endpoint"] = Value::String(
                    options
                        .authorization_endpoint
                        .unwrap_or_else(|| format!("{}authorize", state.issuer)),
                );
            }
            if options.include_jwks_endpoint {
                value["jwks_uri"] = Value::String(
                    options
                        .jwks_endpoint
                        .unwrap_or_else(|| format!("{}jwks", state.issuer)),
                );
            }
            if options.include_token_endpoint {
                value["token_endpoint"] = Value::String(
                    options
                        .token_endpoint
                        .unwrap_or_else(|| format!("{}token", state.issuer)),
                );
            }
            if let Some(methods) = options.auth_methods {
                value["token_endpoint_auth_methods_supported"] = json!(methods);
            }
            if let Some(grants) = options.grant_types {
                value["grant_types_supported"] = json!(grants);
            }
            Json(value).into_response()
        }
    }
}

async fn authorize(Query(query): Query<HashMap<String, String>>) -> Response {
    let Some(redirect_uri) = query.get("redirect_uri") else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if query.get("response_type").map(String::as_str) != Some("code")
        || query.get("scope").is_none_or(|scope| {
            !scope
                .split_ascii_whitespace()
                .any(|scope| scope == "openid")
        })
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Ok(mut location) = url::Url::parse(redirect_uri) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    location
        .query_pairs_mut()
        .append_pair("code", "provider-code");
    if let Some(state) = query.get("state") {
        location.query_pairs_mut().append_pair("state", state);
    }
    (
        StatusCode::FOUND,
        [(header::LOCATION, location.to_string())],
    )
        .into_response()
}

async fn jwks(State(state): State<Arc<ProviderState>>) -> Response {
    state.counters.jwks.fetch_add(1, Ordering::SeqCst);
    match state.options.lock().await.jwks_mode.clone() {
        JwksMode::Key(key_id) => Json(jwks_value(&key_id)).into_response(),
        JwksMode::Empty => Json(json!({ "keys": [] })).into_response(),
        JwksMode::Unusable => Json(json!({
            "keys": [{ "kty": "unsupported", "kid": "bad", "use": "enc" }]
        }))
        .into_response(),
        JwksMode::ServerUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "provider temporarily unavailable",
        )
            .into_response(),
        JwksMode::Redirect => (
            StatusCode::FOUND,
            [(header::LOCATION, format!("{}redirect-target", state.issuer))],
        )
            .into_response(),
    }
}

async fn redirect_target(State(state): State<Arc<ProviderState>>) -> StatusCode {
    state
        .counters
        .redirect_target
        .fetch_add(1, Ordering::SeqCst);
    StatusCode::NO_CONTENT
}

async fn token(
    State(state): State<Arc<ProviderState>>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    state.counters.token.fetch_add(1, Ordering::SeqCst);
    state.token_requests.lock().await.push(TokenRequest {
        authorization: headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned),
        form: form.clone(),
    });
    let barrier = state.token_barrier.lock().await.clone();
    if let Some(barrier) = barrier {
        barrier.wait().await;
    }
    let mut options = state.options.lock().await.clone();
    let expected_login = match form.get("code") {
        Some(code) => state.expected_logins.lock().await.get(code).cloned(),
        None => None,
    };
    if let Some((nonce, challenge)) = expected_login {
        options.expected_nonce = Some(nonce);
        options.expected_challenge = Some(challenge);
    }
    if !valid_token_request(&options, &headers, &form) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant" })),
        )
            .into_response();
    }
    match options.token_mode {
        TokenMode::Malformed => {
            ([(header::CONTENT_TYPE, "application/json")], "{not-json").into_response()
        }
        TokenMode::ProviderError => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant", "error_description": "do not expose" })),
        )
            .into_response(),
        TokenMode::ServerUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "provider temporarily unavailable",
        )
            .into_response(),
        TokenMode::Redirect => (
            StatusCode::FOUND,
            [(header::LOCATION, format!("{}redirect-target", state.issuer))],
        )
            .into_response(),
        TokenMode::MissingIdToken => Json(json!({
            "access_token": "access-token",
            "token_type": "Bearer"
        }))
        .into_response(),
        mode => {
            let nonce = options
                .expected_nonce
                .as_deref()
                .unwrap_or("missing-expected-nonce");
            let token = if matches!(mode, TokenMode::MalformedIdToken) {
                String::from("not-a-compact-id-token")
            } else {
                signed_token(&state.issuer, &options.signing_key_id, nonce, &mode)
            };
            let serialized = if matches!(mode, TokenMode::CorruptSignature) {
                corrupt_signature(&token)
            } else {
                token
            };
            state.issued_id_tokens.lock().await.push(serialized.clone());
            Json(json!({
                "access_token": "access-token",
                "token_type": "Bearer",
                "id_token": serialized,
            }))
            .into_response()
        }
    }
}

fn valid_token_request(
    options: &ProviderOptions,
    headers: &HeaderMap,
    form: &HashMap<String, String>,
) -> bool {
    let client_auth_valid = options.auth_methods.as_ref().is_none_or(|methods| {
        if methods.iter().any(|method| method == "client_secret_basic") {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                == Some("Basic Z2F6ZWwtY2xpZW50OmdhemVsLXNlY3JldA==")
                && !form.contains_key("client_id")
                && !form.contains_key("client_secret")
        } else {
            !headers.contains_key(header::AUTHORIZATION)
                && form.get("client_id").map(String::as_str) == Some(CLIENT_ID)
                && form.get("client_secret").map(String::as_str) == Some(CLIENT_SECRET)
        }
    });
    let challenge_valid = options.expected_challenge.as_ref().is_none_or(|expected| {
        form.get("code_verifier").is_some_and(|verifier| {
            openidconnect::PkceCodeChallenge::from_code_verifier_sha256(
                &openidconnect::PkceCodeVerifier::new(verifier.clone()),
            )
            .as_str()
                == expected
        })
    });
    client_auth_valid
        && challenge_valid
        && form.get("grant_type").map(String::as_str) == Some("authorization_code")
        && form.get("redirect_uri").map(String::as_str)
            == Some("https://gazel.example/auth/callback")
}

fn signed_token(issuer: &str, key_id: &str, nonce: &str, mode: &TokenMode) -> String {
    let issuer = if matches!(mode, TokenMode::WrongIssuer) {
        "https://wrong-issuer.example"
    } else {
        issuer
    };
    let audience = if matches!(mode, TokenMode::WrongAudience) {
        "another-client"
    } else {
        CLIENT_ID
    };
    let expiration = if matches!(mode, TokenMode::Expired) {
        chrono::Utc::now() - chrono::Duration::minutes(1)
    } else {
        chrono::Utc::now() + chrono::Duration::minutes(5)
    };
    let nonce = if matches!(mode, TokenMode::WrongNonce) {
        "wrong-nonce"
    } else {
        nonce
    };
    let audiences = if matches!(mode, TokenMode::AdditionalAudience) {
        vec![
            Audience::new(audience.to_string()),
            Audience::new(String::from("untrusted-client")),
        ]
    } else {
        vec![Audience::new(audience.to_string())]
    };
    let claims = CoreIdTokenClaims::new(
        IssuerUrl::new(issuer.to_string()).expect("issuer should parse"),
        audiences,
        expiration,
        chrono::Utc::now(),
        StandardClaims::new(SubjectIdentifier::new(String::from("test-subject"))),
        EmptyAdditionalClaims {},
    );
    let claims = if matches!(mode, TokenMode::MissingNonce) {
        claims
    } else {
        claims.set_nonce(Some(Nonce::new(nonce.to_string())))
    };
    let key = signing_key(key_id);
    let hash_token = if matches!(mode, TokenMode::InvalidAccessTokenHash) {
        AccessToken::new(String::from("different-access-token"))
    } else {
        AccessToken::new(String::from("access-token"))
    };
    CoreIdToken::new(
        claims,
        &key,
        CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
        Some(&hash_token),
        None,
    )
    .expect("test ID token should sign")
    .to_string()
}

fn corrupt_signature(token: &str) -> String {
    let mut corrupted = token.as_bytes().to_vec();
    let signature_start = token
        .rfind('.')
        .expect("signed token should contain a signature")
        + 1;
    let first = corrupted
        .get_mut(signature_start)
        .expect("signature should not be empty");
    *first = if *first == b'A' { b'B' } else { b'A' };
    String::from_utf8(corrupted).expect("token should remain ASCII")
}

fn signing_key(key_id: &str) -> CoreRsaPrivateSigningKey {
    CoreRsaPrivateSigningKey::from_pem(RSA_PRIVATE_KEY, Some(JsonWebKeyId::new(key_id.to_string())))
        .expect("test RSA key should parse")
}

fn jwks_value(key_id: &str) -> Value {
    serde_json::to_value(openidconnect::core::CoreJsonWebKeySet::new(vec![
        signing_key(key_id).as_verification_key(),
    ]))
    .expect("JWKS should serialize")
}

#[derive(Debug)]
struct LoginAttempt {
    cookie: String,
    state: String,
    nonce: String,
    challenge: String,
}

async fn login_attempt(app: &Router, provider: &MockProvider, return_to: &str) -> LoginAttempt {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/auth/login?return_to={return_to}"))
                .header("forwarded", "host=attacker.example;proto=http")
                .header("x-forwarded-host", "attacker.example")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("login request should succeed");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cookie = response_cookie(&response);
    let location = response_location(&response);
    let url = url::Url::parse(&location).expect("authorization location should be absolute");
    assert_eq!(
        url.origin().ascii_serialization(),
        provider.issuer().trim_end_matches('/')
    );
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
    assert_eq!(query.get("scope").map(String::as_str), Some("openid"));
    assert_eq!(query.get("client_id").map(String::as_str), Some(CLIENT_ID));
    assert_eq!(
        query.get("redirect_uri").map(String::as_str),
        Some("https://gazel.example/auth/callback")
    );
    assert_eq!(
        query.get("code_challenge_method").map(String::as_str),
        Some("S256")
    );
    let state = query.get("state").expect("state should exist").clone();
    let nonce = query.get("nonce").expect("nonce should exist").clone();
    let challenge = query
        .get("code_challenge")
        .expect("challenge should exist")
        .clone();
    provider
        .expect_login(state.clone(), nonce.clone(), challenge.clone())
        .await;
    LoginAttempt {
        cookie,
        state,
        nonce,
        challenge,
    }
}

async fn callback_request(app: &Router, attempt: &LoginAttempt, suffix: &str) -> Response {
    callback_request_with_code(app, attempt, &attempt.state, suffix).await
}

async fn callback_request_with_code(
    app: &Router,
    attempt: &LoginAttempt,
    code: &str,
    suffix: &str,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/auth/callback?code={code}&state={}{}",
                    attempt.state, suffix
                ))
                .header(header::COOKIE, &attempt.cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("callback should succeed")
}

fn response_set_cookie(response: &Response) -> &str {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("session cookie should be present")
        .to_str()
        .expect("cookie should be ASCII")
}

fn response_cookie(response: &Response) -> String {
    response_set_cookie(response)
        .split(';')
        .next()
        .expect("cookie pair should exist")
        .to_string()
}

fn cookie_max_age(set_cookie: &str) -> i64 {
    set_cookie
        .split(';')
        .map(str::trim)
        .find_map(|attribute| attribute.strip_prefix("Max-Age="))
        .expect("cookie should include Max-Age")
        .parse()
        .expect("Max-Age should be an integer")
}

fn tamper_cookie(cookie: &str) -> String {
    let mut bytes = cookie.as_bytes().to_vec();
    let last = bytes.last_mut().expect("cookie should not be empty");
    *last = if *last == b'A' { b'B' } else { b'A' };
    String::from_utf8(bytes).expect("cookie should remain ASCII")
}

fn response_location(response: &Response) -> String {
    response
        .headers()
        .get(header::LOCATION)
        .expect("Location should be present")
        .to_str()
        .expect("Location should be ASCII")
        .to_string()
}

async fn initialized(options: ProviderOptions) -> (MockProvider, Arc<Authentication>, Router) {
    let provider = MockProvider::start(options).await;
    let authentication = Arc::new(
        Authentication::bootstrap(provider.config())
            .await
            .expect("provider should initialize"),
    );
    let app = authentication.protocol_router();
    (provider, authentication, app)
}

async fn initialized_with_clock(
    options: ProviderOptions,
    clock: Arc<TestClock>,
) -> (MockProvider, Router) {
    let provider = MockProvider::start(options).await;
    let authentication = Arc::new(
        Authentication::bootstrap_with_clock(provider.config(), clock)
            .await
            .expect("provider should initialize"),
    );
    let app = authentication.protocol_router();
    (provider, app)
}

async fn server_app(authentication: Option<Authentication>) -> Router {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("test database should connect");
    gazel::server::router(gazel::state::AppState::new(pool, authentication))
}

async fn initialized_server(options: ProviderOptions) -> (MockProvider, Router) {
    let provider = MockProvider::start(options).await;
    let authentication = Authentication::bootstrap(provider.config())
        .await
        .expect("provider should initialize");
    let app = server_app(Some(authentication)).await;
    (provider, app)
}

async fn response_json(response: Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response should contain JSON")
}

async fn response_text(response: Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("response should contain UTF-8")
}

async fn assert_callback_completion(response: Response, return_to: &str) {
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(header::LOCATION).is_none());
    let body = response_text(response).await;
    let script_destination =
        serde_json::to_string(return_to).expect("return target should serialize");
    assert!(
        body.contains(&format!("location.replace({script_destination})")),
        "completion script should navigate to {return_to:?}: {body}"
    );
    assert!(
        body.contains(&format!("href=\"{return_to}\"")),
        "completion fallback should link to {return_to:?}: {body}"
    );
}

#[tokio::test]
async fn mock_provider_runs_entirely_on_loopback() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let issuer = url::Url::parse(provider.issuer()).expect("issuer should parse");
    assert_eq!(issuer.host_str(), Some("127.0.0.1"));
    Authentication::bootstrap(provider.config())
        .await
        .expect("loopback discovery and JWKS should work");
    assert_eq!(provider.discovery_count(), 1);
    assert_eq!(provider.jwks_count(), 1);
}

#[tokio::test]
async fn mock_authorization_endpoint_returns_a_standards_shaped_callback() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let client = openidconnect::reqwest::Client::builder()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .expect("test client should build");
    let response = client
        .get(format!("{}authorize", provider.issuer()))
        .query(&[
            ("response_type", "code"),
            ("scope", "openid"),
            ("state", "test-state"),
            ("redirect_uri", "https://gazel.example/auth/callback"),
        ])
        .send()
        .await
        .expect("authorization request should succeed");
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("https://gazel.example/auth/callback?code=provider-code&state=test-state")
    );
}

#[tokio::test]
async fn discovery_preserves_exact_origin_only_issuer_identifier() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let provider_issuer = provider.issuer().trim_end_matches('/').to_owned();
    provider
        .set_options(|options| options.issuer_override = Some(provider_issuer.clone()))
        .await;

    let mut exact_config = provider.config();
    exact_config.issuer.clone_from(&provider_issuer);
    Authentication::bootstrap(exact_config)
        .await
        .expect("matching origin-only issuer should initialize");

    let error = Authentication::bootstrap(provider.config())
        .await
        .expect_err("trailing slash must fail exact issuer validation");
    assert!(error.to_string().contains("discovery"));
    assert_eq!(provider.discovery_count(), 2);
    assert_eq!(
        provider.jwks_count(),
        1,
        "issuer mismatch must fail before JWKS"
    );
}

#[tokio::test]
async fn discovery_accepts_basic_post_omitted_and_combined_methods() {
    for methods in [
        Some(vec!["client_secret_basic"]),
        Some(vec!["client_secret_post"]),
        None,
        Some(vec!["client_secret_post", "client_secret_basic"]),
    ] {
        let options = ProviderOptions {
            auth_methods: methods.map(|values| values.into_iter().map(String::from).collect()),
            ..ProviderOptions::default()
        };
        let provider = MockProvider::start(options).await;
        Authentication::bootstrap(provider.config())
            .await
            .expect("supported metadata should initialize");
    }
}

#[tokio::test]
async fn discovery_rejects_unsupported_client_authentication() {
    let options = ProviderOptions {
        auth_methods: Some(vec![String::from("private_key_jwt")]),
        ..ProviderOptions::default()
    };
    let provider = MockProvider::start(options).await;
    let error = Authentication::bootstrap(provider.config())
        .await
        .expect_err("unsupported client authentication should fail");
    assert!(error.to_string().contains("client authentication method"));
}

#[tokio::test]
async fn discovery_fails_closed_for_redirects_malformed_metadata_and_wrong_issuer() {
    for options in [
        ProviderOptions {
            metadata_mode: MetadataMode::Redirect,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            metadata_mode: MetadataMode::Malformed,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            issuer_override: Some(String::from("https://wrong.example")),
            ..ProviderOptions::default()
        },
        ProviderOptions {
            jwks_mode: JwksMode::Redirect,
            ..ProviderOptions::default()
        },
    ] {
        let provider = MockProvider::start(options).await;
        assert!(Authentication::bootstrap(provider.config()).await.is_err());
        assert_eq!(
            provider.redirect_count(),
            0,
            "redirects must not be followed"
        );
    }
}

#[tokio::test]
async fn discovery_rejects_insecure_or_missing_endpoints_and_code_flow() {
    let cases = [
        ProviderOptions {
            authorization_endpoint: Some(String::from("http://example.com/authorize")),
            ..ProviderOptions::default()
        },
        ProviderOptions {
            token_endpoint: Some(String::from("http://example.com/token")),
            ..ProviderOptions::default()
        },
        ProviderOptions {
            jwks_endpoint: Some(String::from("http://example.com/jwks")),
            ..ProviderOptions::default()
        },
        ProviderOptions {
            token_endpoint: Some(String::new()),
            ..ProviderOptions::default()
        },
        ProviderOptions {
            include_authorization_endpoint: false,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            include_token_endpoint: false,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            include_jwks_endpoint: false,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            response_types: vec![String::from("id_token")],
            ..ProviderOptions::default()
        },
        ProviderOptions {
            grant_types: Some(vec![String::from("implicit")]),
            ..ProviderOptions::default()
        },
    ];
    for options in cases {
        let provider = MockProvider::start(options).await;
        assert!(Authentication::bootstrap(provider.config()).await.is_err());
    }
}

#[tokio::test]
async fn discovery_rejects_empty_unusable_or_unadvertised_signing_keys() {
    for options in [
        ProviderOptions {
            jwks_mode: JwksMode::Empty,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            jwks_mode: JwksMode::Unusable,
            ..ProviderOptions::default()
        },
        ProviderOptions {
            signing_algorithms: vec![String::from("ES256")],
            ..ProviderOptions::default()
        },
        ProviderOptions {
            signing_algorithms: Vec::new(),
            ..ProviderOptions::default()
        },
    ] {
        let provider = MockProvider::start(options).await;
        let error = Authentication::bootstrap(provider.config())
            .await
            .expect_err("unusable signing keys should fail");
        assert!(error.to_string().contains("signing key"));
    }
}

#[tokio::test]
async fn login_uses_fresh_state_nonce_s256_pkce_and_configured_callback() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let first = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata%23export").await;
    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .header(header::COOKIE, &first.cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("second login should succeed");
    let second_location = response_location(&second_response);
    let second_url = url::Url::parse(&second_location).expect("location should parse");
    let second_query: HashMap<_, _> = second_url.query_pairs().into_owned().collect();
    assert_ne!(second_query.get("state"), Some(&first.state));
    assert_ne!(second_query.get("nonce"), Some(&first.nonce));
    assert_ne!(second_query.get("code_challenge"), Some(&first.challenge));

    let replaced = callback_request(&app, &first, "").await;
    assert_eq!(
        response_location(&replaced),
        "/login?error=authentication_failed&return_to=%2F"
    );
}

#[tokio::test]
async fn successful_callback_uses_a_non_cacheable_completion_document() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata%23export").await;
    let response = callback_request(&app, &attempt, "").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(header::LOCATION).is_none());
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        response
            .headers()
            .get("referrer-policy")
            .and_then(|value| value.to_str().ok()),
        Some("no-referrer")
    );
    let cookie_attributes: Vec<_> = response_set_cookie(&response)
        .split(';')
        .map(str::trim)
        .collect();
    assert!(cookie_attributes.contains(&"SameSite=Lax"));

    let body = response_text(response).await;
    assert!(body.contains("location.replace(\"/settings?tab=data#export\")"));
    assert!(body.contains("<noscript>"));
    assert!(body.contains("href=\"/settings?tab=data#export\""));
}

#[tokio::test]
async fn callback_completion_escapes_the_target_and_excludes_authentication_material() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let encoded_target =
        "%2Fsettings%3Fvalue%3D%22%27%3C%2Fscript%3E%26line%3D%E2%80%A8%E2%80%A9%23fragment";
    let target = "/settings?value=\"'</script>&line=\u{2028}\u{2029}#fragment";
    let attempt = login_attempt(&app, &provider, encoded_target).await;
    let authorization_code = "authorization-code-material";
    provider
        .expect_login(
            String::from(authorization_code),
            attempt.nonce.clone(),
            attempt.challenge.clone(),
        )
        .await;
    let transaction_cookie = attempt.cookie.clone();
    let state = attempt.state.clone();
    let nonce = attempt.nonce.clone();
    let response = callback_request_with_code(
        &app,
        &attempt,
        authorization_code,
        "&error_description=provider-description-secret",
    )
    .await;
    let authenticated_cookie = response_cookie(&response);
    let token_requests = provider.token_requests().await;
    let token_request = token_requests
        .last()
        .expect("token request should be captured");
    let pkce_verifier = token_request
        .form
        .get("code_verifier")
        .expect("PKCE verifier should be sent");
    let issued_id_tokens = provider.issued_id_tokens().await;
    let id_token = issued_id_tokens
        .last()
        .expect("ID token should be captured");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(header::LOCATION).is_none());
    assert_eq!(
        response
            .headers()
            .get("referrer-policy")
            .and_then(|value| value.to_str().ok()),
        Some("no-referrer")
    );
    let body = response_text(response).await;
    assert!(body.contains(
        "location.replace(\"/settings?value=\\\"\\u0027\\u003c/script\\u003e\\u0026line=\\u2028\\u2029#fragment\")"
    ));
    assert!(body.contains(
        "href=\"/settings?value=&quot;&#39;&lt;/script&gt;&amp;line=\u{2028}\u{2029}#fragment\""
    ));
    assert!(!body.contains(target));
    for secret in [
        authorization_code,
        state.as_str(),
        nonce.as_str(),
        pkce_verifier.as_str(),
        id_token.as_str(),
        CLIENT_SECRET,
        "access-token",
        "test-subject",
        "provider-description-secret",
        transaction_cookie.as_str(),
        authenticated_cookie.as_str(),
    ] {
        assert!(
            !body.contains(secret),
            "completion document exposed authentication material: {secret}"
        );
    }
}

#[tokio::test]
async fn mismatched_pkce_is_rejected_without_an_authenticated_session() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    provider
        .expect_login(
            attempt.state.clone(),
            attempt.nonce.clone(),
            String::from("wrong-challenge"),
        )
        .await;
    let response = callback_request(&app, &attempt, "").await;
    assert_eq!(
        response_location(&response),
        "/login?error=authentication_failed&return_to=%2Fsettings"
    );

    let retry = app
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .header(header::COOKIE, attempt.cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("retry should succeed");
    assert!(response_location(&retry).starts_with(provider.issuer()));
}

#[tokio::test]
async fn login_defaults_invalid_percent_decoded_reserved_and_oversized_targets() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    for target in [
        "https%3A%2F%2Fattacker.example",
        "%2F%2Fattacker.example",
        "%252Fsettings",
        "%2Fx%2F..%2Fauth%2Flogin",
        "%2Ffoo%5Cbar",
        "%2Ffoo%255Cbar",
        "%2Ffoo%0Abar",
        "%2Fapi%2Fvehicles",
        "%GG",
    ] {
        let attempt = login_attempt(&app, &provider, target).await;
        let response = callback_request(&app, &attempt, "").await;
        assert_callback_completion(response, "/").await;
    }
    let boundary_target = format!("/{}", "a".repeat(RETURN_TO_MAX_BYTES - 1));
    let boundary = format!("%2F{}", "a".repeat(RETURN_TO_MAX_BYTES - 1));
    let attempt = login_attempt(&app, &provider, &boundary).await;
    let response = callback_request(&app, &attempt, "").await;
    assert_callback_completion(response, &boundary_target).await;

    let oversized = format!("%2F{}", "a".repeat(RETURN_TO_MAX_BYTES));
    let attempt = login_attempt(&app, &provider, &oversized).await;
    let response = callback_request(&app, &attempt, "").await;
    assert_callback_completion(response, "/").await;
}

#[tokio::test]
async fn valid_callbacks_use_the_discovery_selected_wire_auth_and_pkce() {
    for (methods, expect_basic) in [
        (Some(vec!["client_secret_basic"]), true),
        (Some(vec!["client_secret_post"]), false),
        (None, true),
        (
            Some(vec!["client_secret_post", "client_secret_basic"]),
            true,
        ),
    ] {
        let options = ProviderOptions {
            auth_methods: methods.map(|values| values.into_iter().map(String::from).collect()),
            ..ProviderOptions::default()
        };
        let (provider, _, app) = initialized(options).await;
        let attempt = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata").await;
        let response = callback_request(&app, &attempt, "").await;
        assert_callback_completion(response, "/settings?tab=data").await;
        let requests = provider.token_requests().await;
        let request = requests.last().expect("token request should be captured");
        if expect_basic {
            assert_eq!(
                request.authorization.as_deref(),
                Some("Basic Z2F6ZWwtY2xpZW50OmdhemVsLXNlY3JldA==")
            );
            assert!(!request.form.contains_key("client_id"));
            assert!(!request.form.contains_key("client_secret"));
        } else {
            assert!(request.authorization.is_none());
            assert_eq!(
                request.form.get("client_id").map(String::as_str),
                Some(CLIENT_ID)
            );
            assert_eq!(
                request.form.get("client_secret").map(String::as_str),
                Some(CLIENT_SECRET)
            );
        }
        assert!(request.form.contains_key("code_verifier"));
    }
}

#[tokio::test]
async fn successful_callback_rotates_to_authenticated_session_and_discards_tokens() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let response = callback_request(&app, &attempt, "").await;
    let authenticated_cookie = response_cookie(&response);
    assert_ne!(authenticated_cookie, attempt.cookie);

    let already_authenticated = app
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fvehicles")
                .header(header::COOKIE, authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated login should succeed");
    assert_eq!(response_location(&already_authenticated), "/vehicles");
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn disabled_router_preserves_existing_routes_and_exposes_only_inert_auth_config() {
    let app = server_app(None).await;

    let root = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("root request should succeed");
    assert_eq!(root.status(), StatusCode::OK);

    let auth_config = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/config")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("auth config request should succeed");
    assert_eq!(auth_config.status(), StatusCode::OK);
    assert_eq!(
        response_json(auth_config).await,
        json!({ "enabled": false })
    );

    let info = app
        .oneshot(
            Request::builder()
                .uri("/api/info")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("info request should succeed");
    let info = response_json(info).await;
    assert_eq!(
        info,
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "repository": env!("CARGO_PKG_REPOSITORY"),
            "license": env!("CARGO_PKG_LICENSE"),
        })
    );
}

#[tokio::test]
async fn enabled_router_exposes_only_required_public_resources() {
    let (_, app) = initialized_server(ProviderOptions::default()).await;

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("health request should succeed");
    assert_eq!(health.status(), StatusCode::OK);

    let auth_config = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/config")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("auth config request should succeed");
    assert_eq!(
        response_json(auth_config).await,
        json!({ "enabled": true, "provider_name": "Test Provider" })
    );

    let login_page = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/login")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("login page request should succeed");
    assert_eq!(login_page.status(), StatusCode::OK);
    assert_eq!(
        login_page
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html")
    );

    let public_asset_paths = gazel::embedded::public_asset_paths();
    if public_asset_paths.is_empty() {
        return;
    }

    let public_asset = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/favicon.svg")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("asset request should succeed");
    assert_eq!(public_asset.status(), StatusCode::OK);
    assert_eq!(
        public_asset
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/svg+xml")
    );

    for extension in [".js", ".css"] {
        let path = public_asset_paths
            .iter()
            .find(|path| path.ends_with(extension))
            .expect("login bundle should include JS and CSS assets");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path.as_str())
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("asset request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get(header::LOCATION).is_none());
    }
}

#[tokio::test]
async fn enabled_router_protects_application_and_api_routes() {
    let (_, app) = initialized_server(ProviderOptions::default()).await;

    for (target, expected) in [
        (
            "/settings?tab=data",
            "/login?return_to=%2Fsettings%3Ftab%3Ddata",
        ),
        ("/index.html", "/login?return_to=%2Findex.html"),
        ("/missing.asset", "/login?return_to=%2Fmissing.asset"),
        ("/robots.txt", "/login?return_to=%2Frobots.txt"),
        ("/apiary", "/login?return_to=%2Fapiary"),
        ("/foo%5Cbar", "/login?return_to=%2F"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(target)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("protected request should succeed");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response_location(&response), expected);
    }

    let oversized_target = format!("/{}", "a".repeat(RETURN_TO_MAX_BYTES));
    let oversized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(oversized_target)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("oversized request should succeed");
    assert_eq!(response_location(&oversized), "/login?return_to=%2F");

    for target in ["/api", "/api/vehicles"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(target)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("API request should succeed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().get(header::LOCATION).is_none());
        assert_eq!(
            response_json(response).await,
            json!({
                "code": "AUTHENTICATION_REQUIRED",
                "message": "Authentication is required."
            })
        );
    }

    let callback = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/callback")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("callback request should succeed");
    assert_eq!(
        response_location(&callback),
        "/login?error=authentication_failed&return_to=%2F"
    );
}

#[tokio::test]
async fn enabled_router_accepts_authenticated_ui_and_api_requests() {
    let (provider, app) = initialized_server(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let callback = callback_request(&app, &attempt, "").await;
    let authenticated_cookie = response_cookie(&callback);

    let authenticated_login = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/login")
                .header(header::COOKIE, &authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated login request should succeed");
    assert_eq!(authenticated_login.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&authenticated_login), "/");

    let protected_page = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .header(header::COOKIE, &authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated page request should succeed");
    assert_eq!(protected_page.status(), StatusCode::OK);

    let info = app
        .oneshot(
            Request::builder()
                .uri("/api/info")
                .header(header::COOKIE, authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated info request should succeed");
    assert_eq!(
        response_json(info).await,
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "repository": env!("CARGO_PKG_REPOSITORY"),
            "license": env!("CARGO_PKG_LICENSE"),
            "auth_enabled": true,
        })
    );
}

#[tokio::test]
async fn enabled_public_auth_config_uses_the_default_provider_label_only() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let mut config = provider.config();
    config.provider_name = String::from("OpenID Connect");
    let authentication = Authentication::bootstrap(config)
        .await
        .expect("provider should initialize");
    let app = server_app(Some(authentication)).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/config")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("auth config request should succeed");
    assert_eq!(
        response_json(response).await,
        json!({
            "enabled": true,
            "provider_name": "OpenID Connect",
        })
    );
}

#[tokio::test]
async fn logout_is_idempotent_revokes_the_backend_session_and_expires_the_cookie() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let authentication = Authentication::bootstrap(provider.config())
        .await
        .expect("provider should initialize");
    let app = server_app(Some(authentication)).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let callback = callback_request(&app, &attempt, "").await;
    let authenticated_cookie = response_cookie(&callback);

    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header(header::COOKIE, &authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("logout should succeed");
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&logout), "/login?logged_out=1");
    let removal_cookie = response_set_cookie(&logout);
    assert!(removal_cookie.starts_with("gazel_session="));
    assert!(removal_cookie.contains("Max-Age=0"));
    assert!(removal_cookie.contains("Path=/"));

    let former_cookie = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .header(header::COOKIE, &authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("former-cookie request should succeed");
    assert_eq!(
        response_location(&former_cookie),
        "/login?return_to=%2Fsettings"
    );

    let anonymous = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("anonymous logout should succeed");
    assert_eq!(anonymous.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&anonymous), "/login?logged_out=1");

    let malformed = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header(header::COOKIE, "gazel_session=malformed")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("malformed-cookie logout should succeed");
    assert_eq!(malformed.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&malformed), "/login?logged_out=1");
    assert!(response_set_cookie(&malformed).contains("Max-Age=0"));

    assert_eq!(
        provider.token_count(),
        1,
        "logout must not call the provider"
    );
}

#[tokio::test]
async fn private_session_cookie_has_required_attributes_and_is_opaque() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("login should succeed");
    let set_cookie = response_set_cookie(&response);
    let attributes: Vec<_> = set_cookie.split(';').map(str::trim).collect();
    assert!(attributes[0].starts_with("gazel_session="));
    assert!(attributes.contains(&"HttpOnly"));
    assert!(attributes.contains(&"Secure"));
    assert!(attributes.contains(&"SameSite=Lax"));
    assert!(attributes.contains(&"Path=/"));
    assert!(!set_cookie.contains(CLIENT_SECRET));
    assert!(!set_cookie.contains("access-token"));
    assert!(!set_cookie.contains("test-subject"));

    let mut loopback_config = provider.config();
    loopback_config.external_url =
        url::Url::parse("http://127.0.0.1:4110/").expect("URL should parse");
    loopback_config.callback_url =
        url::Url::parse("http://127.0.0.1:4110/auth/callback").expect("URL should parse");
    let loopback_authentication = Arc::new(
        Authentication::bootstrap(loopback_config)
            .await
            .expect("loopback config should initialize"),
    );
    let loopback_response = loopback_authentication
        .protocol_router()
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("login should succeed");
    let loopback_attributes: Vec<_> = response_set_cookie(&loopback_response)
        .split(';')
        .map(str::trim)
        .collect();
    assert!(!loopback_attributes.contains(&"Secure"));
    assert!(loopback_attributes.contains(&"HttpOnly"));
    assert!(loopback_attributes.contains(&"SameSite=Lax"));
    assert!(loopback_attributes.contains(&"Path=/"));
}

#[tokio::test]
async fn tampered_and_unknown_private_cookies_are_unauthenticated() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let callback = callback_request(&app, &attempt, "").await;
    let authenticated_cookie = response_cookie(&callback);

    for cookie in [
        tamper_cookie(&authenticated_cookie),
        String::from("gazel_session=unknown"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/login?return_to=%2Fsettings")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("login should succeed");
        assert!(response_location(&response).starts_with(provider.issuer()));
    }
}

#[tokio::test]
async fn reconstructed_authentication_rejects_a_pre_restart_session_cookie() {
    let provider = MockProvider::start(ProviderOptions::default()).await;
    let first_authentication = Authentication::bootstrap(provider.config())
        .await
        .expect("provider should initialize");
    let first_app = server_app(Some(first_authentication)).await;
    let attempt = login_attempt(&first_app, &provider, "%2Fsettings").await;
    let callback = callback_request(&first_app, &attempt, "").await;
    let pre_restart_cookie = response_cookie(&callback);

    let replacement_authentication = Authentication::bootstrap(provider.config())
        .await
        .expect("provider should reinitialize");
    let replacement_app = server_app(Some(replacement_authentication)).await;
    let response = replacement_app
        .oneshot(
            Request::builder()
                .uri("/settings")
                .header(header::COOKIE, pre_restart_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("post-restart request should succeed");

    assert_eq!(response_location(&response), "/login?return_to=%2Fsettings");
}

#[tokio::test]
async fn session_cookie_expiry_is_five_minutes_then_twelve_non_sliding_hours() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("login should succeed");
    let transaction_cookie = response_cookie(&login);
    assert!((295..=300).contains(&cookie_max_age(response_set_cookie(&login))));

    let location = response_location(&login);
    let authorization_url = url::Url::parse(&location).expect("authorization URL should parse");
    let query: HashMap<_, _> = authorization_url.query_pairs().into_owned().collect();
    let state = query.get("state").expect("state should exist").clone();
    let nonce = query.get("nonce").expect("nonce should exist").clone();
    let challenge = query
        .get("code_challenge")
        .expect("challenge should exist")
        .clone();
    provider.expect_login(state.clone(), nonce, challenge).await;
    let attempt = LoginAttempt {
        cookie: transaction_cookie,
        state,
        nonce: String::new(),
        challenge: String::new(),
    };
    let callback = callback_request(&app, &attempt, "").await;
    assert!((43_195..=43_200).contains(&cookie_max_age(response_set_cookie(&callback))));
    let authenticated_cookie = response_cookie(&callback);
    assert_callback_completion(callback, "/settings").await;

    let authenticated = app
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .header(header::COOKIE, authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated request should succeed");
    assert_eq!(response_location(&authenticated), "/settings");
    assert!(authenticated.headers().get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn injected_clock_rejects_expired_transactions_and_sessions() {
    let clock = Arc::new(TestClock::new(OffsetDateTime::now_utc()));
    let (provider, app) =
        initialized_with_clock(ProviderOptions::default(), Arc::clone(&clock)).await;

    let expired_attempt = login_attempt(&app, &provider, "%2Fexpired").await;
    clock.advance(Duration::minutes(5));
    let expired = callback_request(&app, &expired_attempt, "").await;
    assert_eq!(
        response_location(&expired),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.token_count(), 0);

    let valid_attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let callback = callback_request(&app, &valid_attempt, "").await;
    let authenticated_cookie = response_cookie(&callback);
    assert_callback_completion(callback, "/settings").await;

    clock.advance(Duration::hours(11));
    let before_expiry = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .header(header::COOKIE, &authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("authenticated request should succeed");
    assert_eq!(response_location(&before_expiry), "/settings");
    assert!(before_expiry.headers().get(header::SET_COOKIE).is_none());

    clock.advance(Duration::hours(1));
    let at_expiry = app
        .oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fsettings")
                .header(header::COOKIE, authenticated_cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("expired request should succeed");
    assert!(response_location(&at_expiry).starts_with(provider.issuer()));
}

#[tokio::test]
async fn callbacks_reject_missing_mismatched_and_replayed_state_before_exchange() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let missing = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/callback?code=test-code")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("callback should succeed");
    assert_eq!(
        response_location(&missing),
        "/login?error=authentication_failed&return_to=%2F"
    );

    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let mismatch = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/callback?code=test-code&state=wrong")
                .header(header::COOKIE, &attempt.cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("callback should succeed");
    assert_eq!(
        response_location(&mismatch),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.token_count(), 0);

    let success = callback_request(&app, &attempt, "").await;
    assert_callback_completion(success, "/settings").await;
    let replay = callback_request(&app, &attempt, "").await;
    assert_eq!(
        response_location(&replay),
        "/login?error=authentication_failed&return_to=%2F"
    );

    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn concurrent_callback_consumption_allows_one_token_exchange() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = Arc::new(login_attempt(&app, &provider, "%2Fsettings").await);
    let first = callback_request(&app, &attempt, "");
    let second = callback_request(&app, &attempt, "");
    let (first, second) = tokio::join!(first, second);
    let (success, rejected) = if first.status() == StatusCode::OK {
        (first, second)
    } else {
        (second, first)
    };
    assert_callback_completion(success, "/settings").await;
    assert_eq!(
        response_location(&rejected),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.token_count(), 1);
}

#[tokio::test]
async fn provider_callback_and_token_failures_use_stable_safe_redirects() {
    let modes = [
        TokenMode::Malformed,
        TokenMode::MalformedIdToken,
        TokenMode::ProviderError,
        TokenMode::MissingNonce,
        TokenMode::WrongNonce,
        TokenMode::WrongIssuer,
        TokenMode::WrongAudience,
        TokenMode::Expired,
        TokenMode::InvalidAccessTokenHash,
        TokenMode::AdditionalAudience,
        TokenMode::MissingIdToken,
        TokenMode::CorruptSignature,
        TokenMode::Redirect,
    ];
    for mode in modes {
        let options = ProviderOptions {
            token_mode: mode.clone(),
            ..ProviderOptions::default()
        };
        let (provider, _, app) = initialized(options).await;
        let attempt = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata").await;
        let response = callback_request(&app, &attempt, "").await;
        assert_eq!(
            response_location(&response),
            "/login?error=authentication_failed&return_to=%2Fsettings%3Ftab%3Ddata",
            "unexpected callback outcome for {mode:?}"
        );
        assert_eq!(provider.redirect_count(), 0);
        let retry = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/login?return_to=%2Fsettings")
                    .header(header::COOKIE, &attempt.cookie)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("retry should succeed");
        assert!(response_location(&retry).starts_with(provider.issuer()));
    }

    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let denied = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/auth/callback?error=access_denied&error_description=secret&state={}",
                    attempt.state
                ))
                .header(header::COOKIE, attempt.cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("provider denial should succeed");
    assert_eq!(
        response_location(&denied),
        "/login?error=authentication_failed&return_to=%2Fsettings"
    );
    assert_eq!(provider.token_count(), 0);
}

#[tokio::test]
async fn unavailable_token_transport_uses_provider_unavailable_redirect() {
    let options = ProviderOptions {
        token_endpoint: Some(String::from("http://127.0.0.1:9/token")),
        ..ProviderOptions::default()
    };
    let (provider, _, app) = initialized(options).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let response = callback_request(&app, &attempt, "").await;
    assert_eq!(
        response_location(&response),
        "/login?error=provider_unavailable&return_to=%2Fsettings"
    );
}

#[tokio::test]
async fn token_endpoint_server_error_uses_provider_unavailable_redirect() {
    let options = ProviderOptions {
        token_mode: TokenMode::ServerUnavailable,
        ..ProviderOptions::default()
    };
    let (provider, _, app) = initialized(options).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let response = callback_request(&app, &attempt, "").await;
    assert_eq!(
        response_location(&response),
        "/login?error=provider_unavailable&return_to=%2Fsettings"
    );
}

#[tokio::test]
async fn normal_and_non_signature_failures_use_startup_cache_without_refetch() {
    for mode in [
        TokenMode::Valid,
        TokenMode::MissingNonce,
        TokenMode::WrongNonce,
        TokenMode::InvalidAccessTokenHash,
    ] {
        let options = ProviderOptions {
            token_mode: mode,
            ..ProviderOptions::default()
        };
        let (provider, _, app) = initialized(options).await;
        let attempt = login_attempt(&app, &provider, "%2F").await;
        let _response = callback_request(&app, &attempt, "").await;
        assert_eq!(provider.discovery_count(), 1);
        assert_eq!(provider.jwks_count(), 1);
    }
}

#[tokio::test]
async fn failed_signature_retries_once_with_one_new_generation() {
    let options = ProviderOptions {
        token_mode: TokenMode::CorruptSignature,
        ..ProviderOptions::default()
    };
    let (provider, _, app) = initialized(options).await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let response = callback_request(&app, &attempt, "").await;
    assert_eq!(
        response_location(&response),
        "/login?error=authentication_failed&return_to=%2Fsettings"
    );
    assert_eq!(provider.discovery_count(), 1);
    assert_eq!(provider.jwks_count(), 2);
}

#[tokio::test]
async fn concurrent_unusable_jwks_refresh_is_attempted_once_for_stale_generation() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let first = Arc::new(login_attempt(&app, &provider, "%2Ffirst").await);
    let second = Arc::new(login_attempt(&app, &provider, "%2Fsecond").await);
    provider
        .set_options(|options| {
            options.signing_key_id = String::from("key-2");
            options.jwks_mode = JwksMode::Empty;
        })
        .await;
    provider.set_barrier(2).await;

    let (first_response, second_response) = tokio::join!(
        callback_request(&app, &first, ""),
        callback_request(&app, &second, "")
    );
    for response in [first_response, second_response] {
        assert!(
            response_location(&response)
                .starts_with("/login?error=provider_unavailable&return_to=")
        );
    }
    assert_eq!(provider.jwks_count(), 2, "startup plus one failed refresh");
}

#[tokio::test]
async fn failed_jwks_refresh_retries_after_cooldown_and_recovers() {
    let clock = Arc::new(TestClock::new(OffsetDateTime::now_utc()));
    let (provider, app) =
        initialized_with_clock(ProviderOptions::default(), Arc::clone(&clock)).await;
    provider
        .set_options(|options| {
            options.signing_key_id = String::from("key-2");
            options.jwks_mode = JwksMode::ServerUnavailable;
        })
        .await;

    let failed_attempt = login_attempt(&app, &provider, "%2Ffirst").await;
    let failed_response = callback_request(&app, &failed_attempt, "").await;
    assert_eq!(
        response_location(&failed_response),
        "/login?error=provider_unavailable&return_to=%2Ffirst"
    );
    assert_eq!(provider.jwks_count(), 2, "startup plus failed refresh");

    provider
        .set_options(|options| {
            options.jwks_mode = JwksMode::Key(String::from("key-2"));
        })
        .await;
    let cooldown_attempt = login_attempt(&app, &provider, "%2Fcooldown").await;
    let cooldown_response = callback_request(&app, &cooldown_attempt, "").await;
    assert_eq!(
        response_location(&cooldown_response),
        "/login?error=provider_unavailable&return_to=%2Fcooldown"
    );
    assert_eq!(
        provider.jwks_count(),
        2,
        "the failed generation should not refetch during cooldown"
    );

    clock.advance_utc(Duration::seconds(30));
    let wall_clock_attempt = login_attempt(&app, &provider, "%2Fwall-clock").await;
    let wall_clock_response = callback_request(&app, &wall_clock_attempt, "").await;
    assert_eq!(
        response_location(&wall_clock_response),
        "/login?error=provider_unavailable&return_to=%2Fwall-clock"
    );
    assert_eq!(
        provider.jwks_count(),
        2,
        "wall-clock changes must not release the cooldown"
    );

    clock.advance_monotonic(Duration::seconds(30));
    let first_recovered = Arc::new(login_attempt(&app, &provider, "%2Frecovered-first").await);
    let second_recovered = Arc::new(login_attempt(&app, &provider, "%2Frecovered-second").await);
    provider.set_barrier(2).await;
    let (first_response, second_response) = tokio::join!(
        callback_request(&app, &first_recovered, ""),
        callback_request(&app, &second_recovered, "")
    );
    assert_callback_completion(first_response, "/recovered-first").await;
    assert_callback_completion(second_response, "/recovered-second").await;
    assert_eq!(provider.discovery_count(), 1);
    assert_eq!(
        provider.jwks_count(),
        3,
        "concurrent later logins should share one JWKS retry"
    );
}

#[tokio::test]
async fn concurrent_stale_generation_callbacks_fetch_jwks_once_and_retry_cached_keys() {
    let (provider, _, app) = initialized(ProviderOptions::default()).await;
    let first = Arc::new(login_attempt(&app, &provider, "%2Ffirst").await);
    let second = Arc::new(login_attempt(&app, &provider, "%2Fsecond").await);
    provider
        .set_options(|options| {
            options.signing_key_id = String::from("key-2");
            options.jwks_mode = JwksMode::Key(String::from("key-2"));
        })
        .await;
    provider.set_barrier(2).await;

    let first_callback = callback_request(&app, &first, "");
    let second_callback = callback_request(&app, &second, "");
    let (first_response, second_response) = tokio::join!(first_callback, second_callback);
    assert_callback_completion(first_response, "/first").await;
    assert_callback_completion(second_response, "/second").await;
    assert_eq!(provider.discovery_count(), 1);
    assert_eq!(provider.jwks_count(), 2, "startup plus one refresh");
}
