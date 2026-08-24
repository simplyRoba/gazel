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
use tokio::sync::{Mutex, RwLock};

use crate::config::OidcConfig;

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
    HttpClient(reqwest::Error),
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
            Self::HttpClient(_) => formatter.write_str("failed to build the guarded OIDC client"),
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

impl std::error::Error for OidcStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HttpClient(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ProtocolError {
    Authentication,
    ProviderUnavailable,
}

#[derive(Clone)]
pub(crate) struct GuardedHttpClient {
    inner: reqwest::Client,
}

impl fmt::Debug for GuardedHttpClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GuardedHttpClient").finish()
    }
}

impl GuardedHttpClient {
    fn new() -> Result<Self, reqwest::Error> {
        reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|inner| Self { inner })
    }
}

#[derive(Debug)]
pub(crate) enum GuardedHttpError {
    UnsafeUrl,
    ServerStatus(http::StatusCode),
    Request(Box<reqwest::Error>),
    Http(http::Error),
}

impl fmt::Display for GuardedHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeUrl => formatter.write_str("refused unsafe OIDC transport URL"),
            Self::ServerStatus(status) => write!(formatter, "OIDC server returned {status}"),
            Self::Request(_) => formatter.write_str("OIDC HTTP request failed"),
            Self::Http(_) => formatter.write_str("OIDC HTTP response conversion failed"),
        }
    }
}

impl std::error::Error for GuardedHttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::Http(error) => Some(error),
            Self::UnsafeUrl | Self::ServerStatus(_) => None,
        }
    }
}

impl<'client> AsyncHttpClient<'client> for GuardedHttpClient {
    type Error = GuardedHttpError;
    type Future =
        Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + Sync + 'client>>;

    fn call(&'client self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let url =
                Url::parse(&request.uri().to_string()).map_err(|_| GuardedHttpError::UnsafeUrl)?;
            validate_backend_url(&url).map_err(|()| GuardedHttpError::UnsafeUrl)?;

            let response = self
                .inner
                .execute(
                    request
                        .try_into()
                        .map_err(|error| GuardedHttpError::Request(Box::new(error)))?,
                )
                .await
                .map_err(|error| GuardedHttpError::Request(Box::new(error)))?;
            if response.status().is_server_error() {
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
                        .map_err(|error| GuardedHttpError::Request(Box::new(error)))?
                        .to_vec(),
                )
                .map_err(GuardedHttpError::Http)
        })
    }
}

struct CacheState {
    client: CachedClient,
    generation: u64,
    refresh_failed_for: Option<u64>,
}

pub(crate) struct OidcRuntime {
    metadata: CoreProviderMetadata,
    client_id: ClientId,
    client_secret: ClientSecret,
    callback_url: RedirectUrl,
    auth_method: ClientAuthMethod,
    http: GuardedHttpClient,
    cache: RwLock<CacheState>,
    refresh: Mutex<()>,
}

impl fmt::Debug for OidcRuntime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OidcRuntime")
            .field("issuer", self.metadata.issuer())
            .field(
                "authorization_endpoint",
                self.metadata.authorization_endpoint(),
            )
            .field("token_endpoint", &self.metadata.token_endpoint())
            .field("jwks_uri", self.metadata.jwks_uri())
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("callback_url", &self.callback_url)
            .field("auth_method", &self.auth_method)
            .finish_non_exhaustive()
    }
}

impl OidcRuntime {
    pub(crate) async fn discover(config: &OidcConfig) -> Result<Self, OidcStartupError> {
        let http = GuardedHttpClient::new().map_err(OidcStartupError::HttpClient)?;
        let issuer = IssuerUrl::new(config.issuer.to_string())
            .map_err(|_| OidcStartupError::InvalidIssuer)?;
        let metadata = CoreProviderMetadata::discover_async(issuer, &http)
            .await
            .map_err(|_| OidcStartupError::Discovery)?;

        validate_metadata(&metadata)?;
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
            client_id,
            client_secret,
            callback_url,
            auth_method,
            http,
            cache: RwLock::new(CacheState {
                client,
                generation: 0,
                refresh_failed_for: None,
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
    ) -> Result<String, ProtocolError> {
        let snapshot = self.cache.read().await;
        let client = snapshot.client.clone();
        let generation = snapshot.generation;
        drop(snapshot);

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

        match verify_token(&client, id_token, &token_response, nonce) {
            Ok(subject) => Ok(subject),
            Err(error) if eligible_for_refresh(&error) => {
                let client = self.refresh_for_generation(generation).await?;
                verify_token(&client, id_token, &token_response, nonce)
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
        if current.refresh_failed_for == Some(stale_generation) {
            return Err(ProtocolError::ProviderUnavailable);
        }
        drop(current);

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
        cache.refresh_failed_for = None;
        cache.client = client.clone();
        Ok(client)
    }

    async fn record_refresh_failure(&self, stale_generation: u64) {
        let mut cache = self.cache.write().await;
        if cache.generation == stale_generation {
            cache.refresh_failed_for = Some(stale_generation);
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

fn validate_metadata(metadata: &CoreProviderMetadata) -> Result<(), OidcStartupError> {
    validate_backend_url(metadata.authorization_endpoint().url())
        .map_err(|()| OidcStartupError::UnsafeEndpoint("authorization"))?;
    let token_endpoint = metadata
        .token_endpoint()
        .ok_or(OidcStartupError::MissingTokenEndpoint)?;
    validate_backend_url(token_endpoint.url())
        .map_err(|()| OidcStartupError::UnsafeEndpoint("token"))?;
    validate_backend_url(metadata.jwks_uri().url())
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

fn validate_backend_url(url: &Url) -> Result<(), ()> {
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.host().is_none()
    {
        return Err(());
    }
    match url.scheme() {
        "https" => Ok(()),
        "http" if is_loopback(url) => Ok(()),
        _ => Err(()),
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
    match error {
        openidconnect::RequestTokenError::Request(_) => ProtocolError::ProviderUnavailable,
        _ => ProtocolError::Authentication,
    }
}

fn verify_token(
    client: &CachedClient,
    id_token: &CoreIdToken,
    token_response: &openidconnect::core::CoreTokenResponse,
    nonce: &Nonce,
) -> Result<String, ClaimsVerificationError> {
    let verifier = client.id_token_verifier();
    let claims = id_token.claims(&verifier, nonce)?;
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
        .map_err(|error| ClaimsVerificationError::Other(error.to_string()))?;
        if actual_hash != *expected_hash {
            return Err(ClaimsVerificationError::Other(String::from(
                "invalid access token hash",
            )));
        }
    }
    Ok(claims.subject().to_string())
}

fn eligible_for_refresh(error: &ClaimsVerificationError) -> bool {
    matches!(
        error,
        ClaimsVerificationError::SignatureVerification(
            SignatureVerificationError::NoMatchingKey | SignatureVerificationError::CryptoError(_)
        )
    )
}
