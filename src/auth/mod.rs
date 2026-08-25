mod flow;
mod runtime;
mod session;

use std::fmt;
use std::sync::Arc;

use crate::config::OidcConfig;

pub use flow::Clock;
use flow::{SystemClock, TransactionRegistry};
pub(crate) use flow::{login_redirect, redirect, validate_return_to};
use runtime::{OidcRuntime, OidcStartupError};
use session::SessionBackend;
pub use session::{AuthSessionLayer, CookieKeyError, SESSION_COOKIE_NAME};

/// Error encountered while preparing enabled authentication before serving traffic.
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

/// Process-local state required whenever built-in authentication is enabled.
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

    /// Prepare authentication state with an explicit time source.
    ///
    /// # Errors
    ///
    /// Returns an error when a secure private-cookie key cannot be generated or
    /// the configured OIDC provider cannot be safely initialized.
    #[doc(hidden)]
    pub async fn bootstrap_with_clock(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, AuthenticationStartupError> {
        let sessions = SessionBackend::new(config.secure_cookie())
            .map_err(AuthenticationStartupError::CookieKey)?;
        let runtime = OidcRuntime::discover(&config, Arc::clone(&clock))
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

    /// Validated OIDC configuration retained for startup and request handling.
    pub fn config(&self) -> &OidcConfig {
        &self.config
    }

    /// Build the private, process-local session layer.
    pub fn session_layer(&self) -> AuthSessionLayer {
        self.sessions.layer()
    }

    /// Build the public OIDC protocol routes with their private session layer.
    ///
    /// This is primarily useful for focused protocol tests. The application
    /// server uses one outer session layer around all public and protected
    /// routes instead.
    pub fn protocol_router(self: &Arc<Self>) -> axum::Router {
        flow::routes(self).layer(self.session_layer())
    }

    pub(crate) fn protocol_routes(self: &Arc<Self>) -> axum::Router {
        flow::routes(self)
    }
}
