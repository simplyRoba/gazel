//! Gazel-owned authentication policy and configuration-source adapter.

use super::config::{AuthConfigSource, OidcConfig};
use super::runtime::UrlPolicy;

pub(super) const ENV_PREFIX: &str = "GAZEL_";
pub const SESSION_COOKIE_NAME: &str = "gazel_session";
pub(super) const NON_DOCUMENT_ROOTS: &[&str] = &[];

pub(super) fn config_policy() -> UrlPolicy {
    UrlPolicy::LoopbackHttp
}

pub(super) fn provider_policy(_config: &OidcConfig) -> UrlPolicy {
    UrlPolicy::LoopbackHttp
}

pub(super) fn cookie_secure(config: &OidcConfig) -> bool {
    config.external_url.scheme() == "https"
}

#[cfg(test)]
pub(super) const FIXTURE_APP: &str = "gazel";

pub(crate) struct AuthSource<'a, S>(pub &'a S);

impl<S: crate::config::ConfigSource> AuthConfigSource for AuthSource<'_, S> {
    fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError> {
        self.0.get(&format!("{ENV_PREFIX}{key}"))
    }
}
