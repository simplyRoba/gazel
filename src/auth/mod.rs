pub(crate) mod app;
pub mod config;
mod return_to;
mod routes;
mod runtime;
mod session;
mod transaction;

use std::fmt;
use std::sync::Arc;

use config::OidcConfig;

pub use app::SESSION_COOKIE_NAME;
pub(crate) use return_to::validate_return_to;
pub(crate) use routes::{login_redirect, redirect};
#[doc(hidden)]
pub use runtime::{
    GuardedHttpError, ProviderTransport, ReqwestTransport, TransportFuture, UrlPolicy,
};
use runtime::{OidcRuntime, OidcStartupError};
use session::SessionBackend;
pub use session::{AuthSessionLayer, CookieKeyError};
#[doc(hidden)]
pub use session::{SessionBarrier, SessionFaults, StoreOperation};
pub use transaction::Clock;
use transaction::{SystemClock, TransactionRegistry};

#[derive(Debug)]
pub enum AuthenticationStartupError {
    CookieKey(CookieKeyError),
    Oidc(OidcStartupError),
}

impl fmt::Display for AuthenticationStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CookieKey(error) => error.fmt(formatter),
            Self::Oidc(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AuthenticationStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CookieKey(error) => Some(error),
            Self::Oidc(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub struct Authentication {
    config: OidcConfig,
    sessions: SessionBackend,
    runtime: Arc<OidcRuntime>,
    transactions: Arc<TransactionRegistry>,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Authentication")
            .field("config", &self.config)
            .field("sessions", &self.sessions)
            .field("runtime", &self.runtime)
            .field("transactions", &self.transactions)
            .field("clock", &"Clock")
            .finish()
    }
}

impl Authentication {
    /// Prepare process-local authentication state before the server starts.
    ///
    /// Discovery, endpoint validation, JWKS retrieval, and token authentication
    /// selection all complete before this function returns successfully.
    ///
    /// # Errors
    ///
    /// Returns an error when a secure private-cookie key cannot be generated or
    /// the configured OIDC provider cannot be safely initialized.
    pub async fn bootstrap(config: OidcConfig) -> Result<Self, AuthenticationStartupError> {
        Self::bootstrap_with_clock(config, Arc::new(SystemClock)).await
    }

    /// # Errors
    ///
    /// Returns an error when a secure private-cookie key cannot be generated or
    /// the configured OIDC provider cannot be safely initialized.
    #[doc(hidden)]
    pub async fn bootstrap_with_clock(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, AuthenticationStartupError> {
        Self::bootstrap_with_session_faults(config, clock, SessionFaults::default()).await
    }

    /// # Errors
    /// Returns the same startup errors as ordinary bootstrap.
    #[doc(hidden)]
    pub async fn bootstrap_with_session_faults(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
        faults: SessionFaults,
    ) -> Result<Self, AuthenticationStartupError> {
        let transport = ReqwestTransport::new()
            .map_err(|_| AuthenticationStartupError::Oidc(OidcStartupError::HttpClient))?;
        Self::bootstrap_with_dependencies(config, clock, faults, Arc::new(transport)).await
    }

    /// Injected provider I/O remains behind the common destination guard.
    ///
    /// # Errors
    /// Returns the same startup errors as ordinary bootstrap.
    #[doc(hidden)]
    pub async fn bootstrap_with_dependencies(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
        faults: SessionFaults,
        transport: Arc<dyn ProviderTransport>,
    ) -> Result<Self, AuthenticationStartupError> {
        let sessions = SessionBackend::new(config.secure_cookie(), Arc::clone(&clock), faults)
            .map_err(AuthenticationStartupError::CookieKey)?;
        let runtime = OidcRuntime::discover(&config, Arc::clone(&clock), transport)
            .await
            .map_err(AuthenticationStartupError::Oidc)?;
        Ok(Self {
            config,
            sessions,
            runtime: Arc::new(runtime),
            transactions: Arc::new(TransactionRegistry::default()),
            clock,
        })
    }

    pub fn config(&self) -> &OidcConfig {
        &self.config
    }

    pub fn session_layer(&self) -> AuthSessionLayer {
        self.sessions.layer()
    }

    /// Includes session middleware; do not combine with another session layer.
    pub fn protocol_router(self: &Arc<Self>) -> axum::Router {
        routes::routes(self).layer(self.session_layer())
    }

    /// Routes without session middleware.
    #[doc(hidden)]
    pub fn protocol_routes(self: &Arc<Self>) -> axum::Router {
        routes::routes(self)
    }
}
