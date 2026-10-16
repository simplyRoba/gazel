use openidconnect::core::{
    CoreClient, CoreClientAuthMethod, CoreIdToken, CoreJsonWebKey, CoreJsonWebKeySet,
    CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::http;
use openidconnect::reqwest;
use openidconnect::url::{Host, Url};
use openidconnect::{
    AccessTokenHash, AsyncHttpClient, AuthType, AuthorizationCode, ClaimsVerificationError,
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, HttpRequest,
    HttpResponse, IssuerUrl, JsonWebKey, JsonWebKeyAlgorithm, JsonWebKeyUse, JwsSigningAlgorithm,
    Nonce, OAuth2TokenResponse, PkceCodeVerifier, RedirectUrl, SignatureVerificationError,
    TokenResponse,
};
use std::fmt;
use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use super::Clock;
use super::config::OidcConfig;

const JWKS_REFRESH_COOLDOWN: Duration = Duration::from_secs(30);

type CachedClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientAuthMethod {
    Basic,
    Post,
}

impl ClientAuthMethod {
    fn auth_type(self) -> AuthType {
        match self {
            Self::Basic => AuthType::BasicAuth,
            Self::Post => AuthType::RequestBody,
        }
    }
}

#[derive(Debug)]
pub enum OidcStartupError {
    HttpClient,
    InvalidIssuer,
    Discovery,
    UnsafeEndpoint(&'static str),
    MissingTokenEndpoint,
    MissingCodeFlow,
    UnsupportedClientAuthentication,
    UnusableSigningKeys,
    InvalidCallback,
}

impl fmt::Display for OidcStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpClient => formatter.write_str("failed to build the guarded OIDC client"),
            Self::InvalidIssuer => formatter.write_str("the configured OIDC issuer is invalid"),
            Self::Discovery => formatter.write_str("OIDC discovery or JWKS retrieval failed"),
            Self::UnsafeEndpoint(name) => {
                write!(formatter, "the discovered OIDC {name} endpoint is unsafe")
            }
            Self::MissingTokenEndpoint => {
                formatter.write_str("OIDC discovery omitted the token endpoint")
            }
            Self::MissingCodeFlow => {
                formatter.write_str("OIDC discovery does not support Authorization Code flow")
            }
            Self::UnsupportedClientAuthentication => formatter.write_str(
                "OIDC discovery has no supported token-endpoint client authentication method",
            ),
            Self::UnusableSigningKeys => {
                formatter.write_str("OIDC discovery returned no usable ID-token signing key")
            }
            Self::InvalidCallback => formatter.write_str("the configured OIDC callback is invalid"),
        }
    }
}

impl std::error::Error for OidcStartupError {}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    Authentication,
    ProviderUnavailable,
}

#[derive(Clone)]
pub(crate) struct GuardedHttpClient {
    transport: Arc<dyn ProviderTransport>,
    policy: UrlPolicy,
}

#[doc(hidden)]
pub type TransportFuture<'a> =
    Pin<Box<dyn Future<Output = Result<HttpResponse, GuardedHttpError>> + Send + Sync + 'a>>;

/// Injectable provider I/O, always wrapped in the common destination guard.
#[doc(hidden)]
pub trait ProviderTransport: Send + Sync {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_>;
}

#[doc(hidden)]
pub struct ReqwestTransport(reqwest::Client);

impl ReqwestTransport {
    /// Build the redirect-disabled Rustls provider transport.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built.
    pub fn new() -> Result<Self, reqwest::Error> {
        reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .use_rustls_tls()
            .build()
            .map(Self)
    }
}

impl ProviderTransport for ReqwestTransport {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            let response = self
                .0
                .execute(request.try_into().map_err(|_| GuardedHttpError::Request)?)
                .await
                .map_err(|_| GuardedHttpError::Request)?;
            if response.status().is_server_error() || response.status().is_redirection() {
                return Err(GuardedHttpError::ServerStatus(response.status()));
            }
            let mut builder = http::Response::builder()
                .status(response.status())
                .version(response.version());
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            builder
                .body(
                    response
                        .bytes()
                        .await
                        .map_err(|_| GuardedHttpError::Request)?
                        .to_vec(),
                )
                .map_err(|_| GuardedHttpError::Http)
        })
    }
}

impl fmt::Debug for GuardedHttpClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GuardedHttpClient").finish()
    }
}

impl GuardedHttpClient {
    pub(super) fn new(policy: UrlPolicy, transport: Arc<dyn ProviderTransport>) -> Self {
        Self { transport, policy }
    }

    // http::Uri drops fragments. Check raw metadata URLs before conversion too.
    pub(super) fn validate_raw_url(&self, value: &str) -> Result<(), GuardedHttpError> {
        self.policy
            .validate(value)
            .map(|_| ())
            .map_err(|()| GuardedHttpError::UnsafeUrl)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum GuardedHttpError {
    UnsafeUrl,
    ServerStatus(http::StatusCode),
    Request,
    Http,
}

impl fmt::Display for GuardedHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeUrl => formatter.write_str("refused unsafe OIDC transport URL"),
            Self::ServerStatus(status) => write!(formatter, "OIDC server returned {status}"),
            Self::Request => formatter.write_str("OIDC HTTP request failed"),
            Self::Http => formatter.write_str("OIDC HTTP response conversion failed"),
        }
    }
}

impl std::error::Error for GuardedHttpError {}

fn safe_discovery_error(error: &openidconnect::DiscoveryError<GuardedHttpError>) -> String {
    match error {
        openidconnect::DiscoveryError::Request(GuardedHttpError::ServerStatus(status))
        | openidconnect::DiscoveryError::Response(status, _, _) => format!("HTTP {status}"),
        openidconnect::DiscoveryError::Request(_) => String::from("transport failure"),
        openidconnect::DiscoveryError::Parse(_) => String::from("parsing failure"),
        openidconnect::DiscoveryError::Validation(_) => String::from("validation failure"),
        openidconnect::DiscoveryError::UrlParse(_) => String::from("invalid URL"),
        openidconnect::DiscoveryError::Other(_) => String::from("discovery failure"),
        _ => String::from("OIDC discovery failed"),
    }
}

impl<'client> AsyncHttpClient<'client> for GuardedHttpClient {
    type Error = GuardedHttpError;
    type Future =
        Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + Sync + 'client>>;

    fn call(&'client self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            self.validate_raw_url(&request.uri().to_string())?;

            let response = self.transport.send(request).await?;
            if response.status().is_server_error() || response.status().is_redirection() {
                return Err(GuardedHttpError::ServerStatus(response.status()));
            }
            Ok(response)
        })
    }
}

// http::Uri discards fragments before AsyncHttpClient::call. Inspect the original
// JWKS URL in discovery JSON before the library constructs its initial JWKS request.
pub(super) struct DiscoveryHttpClient<'a> {
    http: &'a GuardedHttpClient,
    metadata_pending: AtomicBool,
}

impl<'a> DiscoveryHttpClient<'a> {
    pub(super) fn new(http: &'a GuardedHttpClient) -> Self {
        Self {
            http,
            metadata_pending: AtomicBool::new(true),
        }
    }
}

impl<'client> AsyncHttpClient<'client> for DiscoveryHttpClient<'client> {
    type Error = GuardedHttpError;
    type Future = TransportFuture<'client>;

    fn call(&'client self, request: HttpRequest) -> Self::Future {
        // Discovery fetches metadata first and JWKS second. Only the metadata's
        // jwks_uri selects a destination; a JWKS extension with that name does not.
        let is_metadata = self.metadata_pending.swap(false, Ordering::AcqRel);
        Box::pin(async move {
            let response = self.http.call(request).await?;
            if is_metadata
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(response.body())
                && let Some(jwks_uri) = value.get("jwks_uri").and_then(serde_json::Value::as_str)
            {
                self.http.validate_raw_url(jwks_uri)?;
            }
            Ok(response)
        })
    }
}

struct CacheState {
    client: CachedClient,
    generation: u64,
    refresh_retry_at: Option<Instant>,
}

pub(crate) struct OidcRuntime {
    metadata: CoreProviderMetadata,
    issuer: String,
    client_id: ClientId,
    client_secret: ClientSecret,
    callback_url: RedirectUrl,
    pub(super) auth_method: ClientAuthMethod,
    http: GuardedHttpClient,
    clock: Arc<dyn Clock>,
    cache: RwLock<CacheState>,
    refresh: Mutex<()>,
}

impl fmt::Debug for OidcRuntime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OidcRuntime")
            .field("auth_method", &self.auth_method)
            .finish_non_exhaustive()
    }
}

impl OidcRuntime {
    pub(crate) async fn discover(
        config: &OidcConfig,
        clock: Arc<dyn Clock>,
        transport: Arc<dyn ProviderTransport>,
    ) -> Result<Self, OidcStartupError> {
        let policy = super::app::provider_policy(config);
        let http = GuardedHttpClient::new(policy, transport);
        let issuer =
            IssuerUrl::new(config.issuer.clone()).map_err(|_| OidcStartupError::InvalidIssuer)?;
        http.validate_raw_url(issuer.as_str())
            .map_err(|_| OidcStartupError::InvalidIssuer)?;
        let metadata =
            CoreProviderMetadata::discover_async(issuer, &DiscoveryHttpClient::new(&http))
                .await
                .map_err(|error| {
                    tracing::error!(
                        error = %safe_discovery_error(&error),
                        "OIDC discovery failed"
                    );
                    OidcStartupError::Discovery
                })?;

        if metadata.issuer().as_str() != config.issuer {
            return Err(OidcStartupError::InvalidIssuer);
        }
        validate_metadata(&metadata, policy)?;
        let auth_method = select_client_auth(metadata.token_endpoint_auth_methods_supported())?;
        let callback_url = RedirectUrl::new(config.callback_url.to_string())
            .map_err(|_| OidcStartupError::InvalidCallback)?;
        let client_id = ClientId::new(config.client_id.clone());
        let client_secret = ClientSecret::new(config.client_secret.clone());
        let client = build_client(
            &metadata,
            &client_id,
            &client_secret,
            &callback_url,
            auth_method,
        );

        Ok(Self {
            metadata,
            issuer: config.issuer.clone(),
            client_id,
            client_secret,
            callback_url,
            auth_method,
            http,
            clock,
            cache: RwLock::new(CacheState {
                client,
                generation: 0,
                refresh_retry_at: None,
            }),
            refresh: Mutex::new(()),
        })
    }

    pub(crate) async fn authorization_url(
        &self,
    ) -> (Url, openidconnect::CsrfToken, Nonce, PkceCodeVerifier) {
        let client = self.cache.read().await.client.clone();
        let (challenge, verifier) = openidconnect::PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                openidconnect::AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                openidconnect::CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();
        (url, state, nonce, verifier)
    }

    pub(crate) async fn exchange_and_verify(
        &self,
        code: String,
        nonce: &Nonce,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<(), ProtocolError> {
        let snapshot = self.cache.read().await;
        let client = snapshot.client.clone();
        let generation = snapshot.generation;
        drop(snapshot);

        self.http
            .validate_raw_url(
                client
                    .token_uri()
                    .ok_or(ProtocolError::ProviderUnavailable)?
                    .as_str(),
            )
            .map_err(|_| ProtocolError::ProviderUnavailable)?;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| ProtocolError::Authentication)?
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.http)
            .await
            .map_err(|error| classify_token_error(&error))?;
        let id_token = token_response
            .id_token()
            .ok_or(ProtocolError::Authentication)?;

        match verify_token(
            &client,
            id_token,
            &token_response,
            nonce,
            &self.issuer,
            &self.client_id,
        ) {
            Ok(()) => Ok(()),
            Err(error) if eligible_for_refresh(&error) => {
                let client = self.refresh_for_generation(generation).await?;
                verify_token(
                    &client,
                    id_token,
                    &token_response,
                    nonce,
                    &self.issuer,
                    &self.client_id,
                )
                .map_err(|_| ProtocolError::Authentication)
            }
            Err(_) => Err(ProtocolError::Authentication),
        }
    }

    async fn refresh_for_generation(
        &self,
        stale_generation: u64,
    ) -> Result<CachedClient, ProtocolError> {
        let _refresh = self.refresh.lock().await;
        let current = self.cache.read().await;
        if current.generation != stale_generation {
            return Ok(current.client.clone());
        }
        if current
            .refresh_retry_at
            .is_some_and(|retry_at| self.clock.monotonic_now() < retry_at)
        {
            return Err(ProtocolError::ProviderUnavailable);
        }
        drop(current);

        if self
            .http
            .validate_raw_url(self.metadata.jwks_uri().as_str())
            .is_err()
        {
            self.record_refresh_failure(stale_generation).await;
            return Err(ProtocolError::ProviderUnavailable);
        }
        let Ok(jwks) = CoreJsonWebKeySet::fetch_async(self.metadata.jwks_uri(), &self.http).await
        else {
            self.record_refresh_failure(stale_generation).await;
            return Err(ProtocolError::ProviderUnavailable);
        };
        if validate_signing_keys(&jwks, self.metadata.id_token_signing_alg_values_supported())
            .is_err()
        {
            self.record_refresh_failure(stale_generation).await;
            return Err(ProtocolError::ProviderUnavailable);
        }
        let metadata = self.metadata.clone().set_jwks(jwks);
        let client = build_client(
            &metadata,
            &self.client_id,
            &self.client_secret,
            &self.callback_url,
            self.auth_method,
        );
        let mut cache = self.cache.write().await;
        cache.generation = cache.generation.saturating_add(1);
        cache.refresh_retry_at = None;
        cache.client = client.clone();
        Ok(client)
    }

    async fn record_refresh_failure(&self, stale_generation: u64) {
        let mut cache = self.cache.write().await;
        if cache.generation == stale_generation {
            cache.refresh_retry_at = Some(self.clock.monotonic_now() + JWKS_REFRESH_COOLDOWN);
        }
    }
}

fn build_client(
    metadata: &CoreProviderMetadata,
    client_id: &ClientId,
    client_secret: &ClientSecret,
    callback_url: &RedirectUrl,
    auth_method: ClientAuthMethod,
) -> CachedClient {
    CoreClient::from_provider_metadata(
        metadata.clone(),
        client_id.clone(),
        Some(client_secret.clone()),
    )
    .set_auth_type(auth_method.auth_type())
    .set_redirect_uri(callback_url.clone())
}

fn validate_metadata(
    metadata: &CoreProviderMetadata,
    policy: UrlPolicy,
) -> Result<(), OidcStartupError> {
    policy
        .validate(metadata.authorization_endpoint().as_str())
        .map_err(|()| OidcStartupError::UnsafeEndpoint("authorization"))?;
    let token_endpoint = metadata
        .token_endpoint()
        .ok_or(OidcStartupError::MissingTokenEndpoint)?;
    policy
        .validate(token_endpoint.as_str())
        .map_err(|()| OidcStartupError::UnsafeEndpoint("token"))?;
    policy
        .validate(metadata.jwks_uri().as_str())
        .map_err(|()| OidcStartupError::UnsafeEndpoint("JWKS"))?;

    let has_code_response = metadata
        .response_types_supported()
        .iter()
        .any(|response_types| {
            response_types.len() == 1 && response_types[0] == CoreResponseType::Code
        });
    let grant_allows_code = metadata
        .grant_types_supported()
        .as_ref()
        .is_none_or(|grant_types| {
            grant_types
                .iter()
                .any(|grant| grant == &openidconnect::core::CoreGrantType::AuthorizationCode)
        });
    if !has_code_response || !grant_allows_code {
        return Err(OidcStartupError::MissingCodeFlow);
    }

    validate_signing_keys(
        metadata.jwks(),
        metadata.id_token_signing_alg_values_supported(),
    )
}

fn validate_signing_keys(
    jwks: &CoreJsonWebKeySet,
    algorithms: &[CoreJwsSigningAlgorithm],
) -> Result<(), OidcStartupError> {
    if algorithms.contains(&CoreJwsSigningAlgorithm::None) {
        return Err(OidcStartupError::UnusableSigningKeys);
    }
    let usable = algorithms.iter().any(|algorithm| {
        jwks.keys()
            .iter()
            .any(|key| key_usable_for_algorithm(key, algorithm))
    });
    if usable {
        Ok(())
    } else {
        Err(OidcStartupError::UnusableSigningKeys)
    }
}

fn key_usable_for_algorithm(key: &CoreJsonWebKey, algorithm: &CoreJwsSigningAlgorithm) -> bool {
    if key.key_use().is_some_and(|usage| !usage.allows_signature()) {
        return false;
    }
    if algorithm.key_type().as_ref() != Some(key.key_type()) {
        return false;
    }
    match key.signing_alg() {
        JsonWebKeyAlgorithm::Unspecified => true,
        JsonWebKeyAlgorithm::Algorithm(key_algorithm) => key_algorithm == algorithm,
        JsonWebKeyAlgorithm::Unsupported => false,
    }
}

fn select_client_auth(
    methods: Option<&Vec<CoreClientAuthMethod>>,
) -> Result<ClientAuthMethod, OidcStartupError> {
    let Some(methods) = methods else {
        return Ok(ClientAuthMethod::Basic);
    };
    if methods.contains(&CoreClientAuthMethod::ClientSecretBasic) {
        Ok(ClientAuthMethod::Basic)
    } else if methods.contains(&CoreClientAuthMethod::ClientSecretPost) {
        Ok(ClientAuthMethod::Post)
    } else {
        Err(OidcStartupError::UnsupportedClientAuthentication)
    }
}

/// Explicit application transport policy; never inferred from request headers.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UrlPolicy {
    HttpsOnly,
    LoopbackHttp,
}

impl UrlPolicy {
    pub(super) fn validate(self, value: &str) -> Result<Url, ()> {
        let url = Url::parse(value).map_err(|_| ())?;
        let has_userinfo = value.split_once("://").is_some_and(|(_, remainder)| {
            let end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
            remainder[..end].contains('@')
        });
        if has_userinfo
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.host().is_none()
        {
            return Err(());
        }
        match url.scheme() {
            "https" => Ok(url),
            "http" if self == Self::LoopbackHttp && is_loopback(&url) => Ok(url),
            _ => Err(()),
        }
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address == Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(address)) => address == Ipv6Addr::LOCALHOST,
        None => false,
    }
}

fn classify_token_error<ErrorType>(
    error: &openidconnect::RequestTokenError<GuardedHttpError, ErrorType>,
) -> ProtocolError
where
    ErrorType: openidconnect::ErrorResponse + 'static,
{
    let category = match error {
        openidconnect::RequestTokenError::ServerResponse(_) => ProtocolError::Authentication,
        // Flattened ID-token deserialization can lose its typed error path. Reparse only
        // the envelope without that field to distinguish an invalid token from service failure.
        openidconnect::RequestTokenError::Parse(_, body)
            if invalid_id_token_in_recognizable_response(body) =>
        {
            ProtocolError::Authentication
        }
        _ => ProtocolError::ProviderUnavailable,
    };
    let status = match error {
        openidconnect::RequestTokenError::Request(GuardedHttpError::ServerStatus(status)) => {
            Some(status.as_u16())
        }
        _ => None,
    };
    tracing::warn!(
        operation = "token",
        ?category,
        status,
        "OIDC provider failure"
    );
    category
}

fn invalid_id_token_in_recognizable_response(body: &[u8]) -> bool {
    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return false;
    };
    let Some(object) = value.as_object_mut() else {
        return false;
    };
    if object.remove("id_token").is_none() {
        return false;
    }
    serde_json::from_value::<openidconnect::core::CoreTokenResponse>(value).is_ok()
}

fn verify_token(
    client: &CachedClient,
    id_token: &CoreIdToken,
    token_response: &openidconnect::core::CoreTokenResponse,
    nonce: &Nonce,
    issuer: &str,
    client_id: &ClientId,
) -> Result<(), ClaimsVerificationError> {
    let verifier = client.id_token_verifier();
    let claims = id_token.claims(&verifier, nonce)?;
    if claims.issuer().as_str() != issuer
        || claims.issue_time().timestamp() <= 0
        || claims
            .authorized_party()
            .is_some_and(|party| party != client_id)
    {
        return Err(ClaimsVerificationError::Other(String::from(
            "invalid claims",
        )));
    }
    if let Some(expected_hash) = claims.access_token_hash() {
        let actual_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token
                .signing_alg()
                .map_err(ClaimsVerificationError::SignatureVerification)?,
            id_token
                .signing_key(&verifier)
                .map_err(ClaimsVerificationError::SignatureVerification)?,
        )
        .map_err(|_| ClaimsVerificationError::Other(String::from("invalid access token hash")))?;
        if actual_hash != *expected_hash {
            return Err(ClaimsVerificationError::Other(String::from(
                "invalid access token hash",
            )));
        }
    }
    Ok(())
}

fn eligible_for_refresh(error: &ClaimsVerificationError) -> bool {
    matches!(
        error,
        ClaimsVerificationError::SignatureVerification(
            SignatureVerificationError::NoMatchingKey | SignatureVerificationError::CryptoError(_)
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CountingTransport(std::sync::atomic::AtomicUsize);

    impl ProviderTransport for CountingTransport {
        fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async { Ok(http::Response::new(Vec::new())) })
        }
    }

    #[tokio::test]
    async fn every_request_is_guarded_before_injected_transport() {
        let transport = Arc::new(CountingTransport(std::sync::atomic::AtomicUsize::new(0)));
        let client = GuardedHttpClient::new(UrlPolicy::LoopbackHttp, transport.clone());
        for url in [
            "http://example.com/jwks",
            "https://user:private-sentinel@example.com/token",
            "http://127.0.0.2/token",
            "ftp://localhost/discovery",
        ] {
            let request = http::Request::builder()
                .method(http::Method::POST)
                .uri(url)
                .body(Vec::new())
                .unwrap();
            assert!(
                matches!(client.call(request).await, Err(GuardedHttpError::UnsafeUrl)),
                "{url}"
            );
        }
        assert_eq!(transport.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(
            client
                .validate_raw_url("http://127.0.0.1/jwks#private-sentinel")
                .is_err()
        );
        for url in [
            "https://example.com/token",
            "http://localhost/jwks",
            "http://127.0.0.1/token",
            "http://[::1]/jwks",
        ] {
            let request = http::Request::builder().uri(url).body(Vec::new()).unwrap();
            assert!(client.call(request).await.is_ok());
        }
        assert_eq!(transport.0.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn discovery_guard_does_not_treat_jwks_extensions_as_metadata() {
        struct MetadataThenKeys(std::sync::atomic::AtomicUsize);

        impl ProviderTransport for MetadataThenKeys {
            fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
                let first = self.0.fetch_add(1, Ordering::SeqCst) == 0;
                Box::pin(async move {
                    let body = if first {
                        serde_json::json!({"jwks_uri": "https://keys.example/jwks"})
                    } else {
                        serde_json::json!({"keys": [], "jwks_uri": "http://unsafe.invalid"})
                    };
                    Ok(http::Response::new(serde_json::to_vec(&body).unwrap()))
                })
            }
        }

        let http = GuardedHttpClient::new(
            UrlPolicy::HttpsOnly,
            Arc::new(MetadataThenKeys(std::sync::atomic::AtomicUsize::new(0))),
        );
        let discovery = DiscoveryHttpClient::new(&http);
        for url in [
            "https://issuer.example/metadata",
            "https://keys.example/jwks",
        ] {
            let request = http::Request::builder().uri(url).body(Vec::new()).unwrap();
            assert!(discovery.call(request).await.is_ok());
        }
    }

    #[tokio::test]
    async fn library_token_and_refresh_requests_cannot_bypass_guard() {
        let transport = Arc::new(CountingTransport(std::sync::atomic::AtomicUsize::new(0)));
        let http = GuardedHttpClient::new(UrlPolicy::LoopbackHttp, transport.clone());
        for destination in [
            "http://example.invalid/provider",
            "https://user:private-sentinel@example.com/provider",
        ] {
            let metadata: CoreProviderMetadata = serde_json::from_value(serde_json::json!({
                "issuer": "https://issuer.example",
                "authorization_endpoint": "https://issuer.example/authorize",
                "token_endpoint": destination,
                "jwks_uri": destination,
                "response_types_supported": ["code"],
                "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": ["RS256"]
            }))
            .unwrap();
            let client = build_client(
                &metadata,
                &ClientId::new(String::from("client")),
                &ClientSecret::new(String::from("private-sentinel")),
                &RedirectUrl::new(String::from("https://app.example/auth/callback")).unwrap(),
                ClientAuthMethod::Basic,
            );
            let error = client
                .exchange_code(AuthorizationCode::new(String::from("private-sentinel")))
                .unwrap()
                .request_async(&http)
                .await
                .unwrap_err();
            assert!(matches!(
                classify_token_error(&error),
                ProtocolError::ProviderUnavailable
            ));
            assert!(
                CoreJsonWebKeySet::fetch_async(metadata.jwks_uri(), &http)
                    .await
                    .is_err()
            );
        }
        assert_eq!(transport.0.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn https_only_policy_rejects_loopback_before_injected_transport() {
        let transport = Arc::new(CountingTransport(std::sync::atomic::AtomicUsize::new(0)));
        let http = GuardedHttpClient::new(UrlPolicy::HttpsOnly, transport.clone());
        for destination in [
            "http://localhost/jwks",
            "http://127.0.0.1/token",
            "http://[::1]/metadata",
        ] {
            let request = http::Request::builder()
                .uri(destination)
                .body(Vec::new())
                .unwrap();
            assert!(matches!(
                http.call(request).await,
                Err(GuardedHttpError::UnsafeUrl)
            ));
        }
        assert_eq!(transport.0.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn initial_metadata_raw_jwks_fragment_is_rejected_before_library_fetch() {
        struct UnsafeMetadata(std::sync::atomic::AtomicUsize);
        impl ProviderTransport for UnsafeMetadata {
            fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Box::pin(async {
                    Ok(http::Response::new(
                        br#"{"jwks_uri":"https://keys.example/jwks#private-sentinel"}"#.to_vec(),
                    ))
                })
            }
        }
        let transport = Arc::new(UnsafeMetadata(std::sync::atomic::AtomicUsize::new(0)));
        let http = GuardedHttpClient::new(UrlPolicy::HttpsOnly, transport.clone());
        let discovery = DiscoveryHttpClient::new(&http);
        let request = http::Request::builder()
            .uri("https://issuer.example/metadata")
            .body(Vec::new())
            .unwrap();
        assert!(matches!(
            discovery.call(request).await,
            Err(GuardedHttpError::UnsafeUrl)
        ));
        assert_eq!(transport.0.load(Ordering::SeqCst), 1);
    }

    fn runtime_with_metadata(
        metadata: CoreProviderMetadata,
        transport: Arc<dyn ProviderTransport>,
    ) -> OidcRuntime {
        let client_id = ClientId::new(String::from("client"));
        let client_secret = ClientSecret::new(String::from("private-sentinel"));
        let callback_url =
            RedirectUrl::new(String::from("https://app.example/auth/callback")).unwrap();
        let auth_method = ClientAuthMethod::Basic;
        let client = build_client(
            &metadata,
            &client_id,
            &client_secret,
            &callback_url,
            auth_method,
        );
        OidcRuntime {
            metadata,
            issuer: String::from("https://issuer.example"),
            client_id,
            client_secret,
            callback_url,
            auth_method,
            http: GuardedHttpClient::new(UrlPolicy::LoopbackHttp, transport),
            clock: Arc::new(super::super::transaction::SystemClock),
            cache: RwLock::new(CacheState {
                client,
                generation: 0,
                refresh_retry_at: None,
            }),
            refresh: Mutex::new(()),
        }
    }

    #[tokio::test]
    async fn runtime_raw_token_and_refresh_urls_are_checked_before_uri_fragment_truncation() {
        let transport = Arc::new(CountingTransport(std::sync::atomic::AtomicUsize::new(0)));
        for destination in [
            "http://127.0.0.1/provider#private-sentinel",
            "http://127.0.0.1/provider#",
            "https://user:private-sentinel@example.com/provider",
            "https://@example.com/provider",
            "ftp://localhost/provider",
            "http://example.invalid/provider",
        ] {
            let metadata = serde_json::from_value(serde_json::json!({
                "issuer": "https://issuer.example",
                "authorization_endpoint": "https://issuer.example/authorize",
                "token_endpoint": destination, "jwks_uri": destination,
                "response_types_supported": ["code"], "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": ["RS256"]
            }))
            .unwrap();
            let runtime = runtime_with_metadata(metadata, transport.clone());
            assert!(
                matches!(
                    runtime
                        .exchange_and_verify(
                            String::from("private-sentinel"),
                            &Nonce::new(String::from("nonce")),
                            PkceCodeVerifier::new("x".repeat(43)),
                        )
                        .await,
                    Err(ProtocolError::ProviderUnavailable)
                ),
                "{destination}"
            );
            assert!(
                matches!(
                    runtime.refresh_for_generation(0).await,
                    Err(ProtocolError::ProviderUnavailable)
                ),
                "{destination}"
            );
            let cache = runtime.cache.read().await;
            assert_eq!(cache.generation, 0);
            assert!(
                cache.refresh_retry_at.is_some(),
                "failed raw refresh enters cooldown"
            );
            assert_eq!(transport.0.load(Ordering::SeqCst), 0, "{destination}");
        }
    }

    #[test]
    fn safe_discovery_error_omits_response_body() {
        let error = openidconnect::DiscoveryError::<GuardedHttpError>::Response(
            http::StatusCode::BAD_REQUEST,
            b"client_secret=do-not-log".to_vec(),
            String::from("unexpected content type"),
        );

        let diagnostic = safe_discovery_error(&error);

        assert_eq!(diagnostic, "HTTP 400 Bad Request");
        assert!(!diagnostic.contains("do-not-log"));
    }

    #[test]
    fn safe_discovery_error_omits_validation_detail() {
        let error = openidconnect::DiscoveryError::<GuardedHttpError>::Validation(String::from(
            "unexpected issuer URI `https://wrong.example` (expected `https://issuer.example`)",
        ));

        let diagnostic = safe_discovery_error(&error);

        assert_eq!(diagnostic, "validation failure");
    }
}
