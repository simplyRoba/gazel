use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use url::{Host, Url};

/// Abstraction over configuration sources, enabling testable config loading
/// without touching real environment variables.
pub trait ConfigSource {
    /// Returns the value for the given key, or `None` if not set.
    ///
    /// # Errors
    ///
    /// Returns `VarError::NotUnicode` when a present value is not valid UTF-8.
    fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError>;
}

/// Reads configuration from environment variables.
pub struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError> {
        match std::env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(error @ std::env::VarError::NotUnicode(_)) => Err(error),
        }
    }
}

/// Validated configuration for optional built-in OIDC authentication.
#[derive(Clone)]
pub struct OidcConfig {
    /// Normalized public origin used for browser redirects.
    pub external_url: Url,
    /// Callback URL registered with the OIDC provider.
    pub callback_url: Url,
    /// OIDC issuer URL used for discovery.
    pub issuer: Url,
    /// Confidential-client identifier.
    pub client_id: String,
    /// Confidential-client secret.
    pub client_secret: String,
    /// Human-readable provider label shown on the login page.
    pub provider_name: String,
}

impl fmt::Debug for OidcConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OidcConfig")
            .field("external_url", &self.external_url)
            .field("callback_url", &self.callback_url)
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("provider_name", &self.provider_name)
            .finish()
    }
}

impl OidcConfig {
    /// Whether the session cookie must carry the `Secure` attribute.
    pub fn secure_cookie(&self) -> bool {
        self.external_url.scheme() == "https"
    }
}

/// Application configuration read from environment variables.
#[derive(Debug)]
pub struct Config {
    /// HTTP server listen port.
    pub port: u16,
    /// Filesystem path to the `SQLite` database.
    pub db_path: String,
    /// Tracing log-level filter.
    pub log_level: String,
    /// Validated OIDC settings, or `None` when authentication is disabled.
    pub auth: Option<OidcConfig>,
}

/// Error returned when an authentication setting is unsafe or incomplete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError(String);

impl ConfigError {
    fn invalid(key: &str, reason: &str) -> Self {
        Self(format!("invalid {key}: {reason}"))
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Load configuration from real environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error when authentication is explicitly enabled but its
    /// boolean flag, required values, URLs, or provider display name are
    /// invalid.
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&EnvConfigSource)
    }

    /// Load configuration from an arbitrary source.
    ///
    /// Legacy port, database, and log settings retain their existing fallback
    /// behavior. Authentication settings fail closed once enabled.
    ///
    /// # Errors
    ///
    /// Returns an error when authentication configuration is malformed,
    /// incomplete, or unsafe.
    pub fn load_from(source: &impl ConfigSource) -> Result<Self, ConfigError> {
        let auth_enabled_value = auth_value(source, "GAZEL_AUTH_ENABLED")?;
        let auth_enabled = match auth_enabled_value.as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(_) => {
                return Err(ConfigError::invalid(
                    "GAZEL_AUTH_ENABLED",
                    "expected exactly `true` or `false`",
                ));
            }
        };

        let auth = auth_enabled.then(|| load_oidc_config(source)).transpose()?;

        Ok(Self {
            port: parse_or(source, "GAZEL_PORT", 4110),
            db_path: parse_or(source, "GAZEL_DB_PATH", String::from("/data/gazel.db")),
            log_level: parse_or(source, "GAZEL_LOG_LEVEL", String::from("info")),
            auth,
        })
    }
}

fn load_oidc_config(source: &impl ConfigSource) -> Result<OidcConfig, ConfigError> {
    let external_value = required(source, "GAZEL_EXTERNAL_URL")?;
    let issuer_value = required(source, "GAZEL_OIDC_ISSUER")?;
    let client_id = required(source, "GAZEL_OIDC_CLIENT_ID")?;
    let client_secret = required(source, "GAZEL_OIDC_CLIENT_SECRET")?;

    let mut external_url = validate_auth_url(&external_value, "GAZEL_EXTERNAL_URL")?;
    if external_url.path() != "/" {
        return Err(ConfigError::invalid(
            "GAZEL_EXTERNAL_URL",
            "must be a root-mounted origin without a path",
        ));
    }
    external_url.set_path("/");

    let callback_url = external_url
        .join("auth/callback")
        .map_err(|_| ConfigError::invalid("GAZEL_EXTERNAL_URL", "cannot construct callback"))?;
    let issuer = validate_auth_url(&issuer_value, "GAZEL_OIDC_ISSUER")?;
    let provider_name = provider_name(source)?;

    Ok(OidcConfig {
        external_url,
        callback_url,
        issuer,
        client_id,
        client_secret,
        provider_name,
    })
}

fn auth_value(source: &impl ConfigSource, key: &str) -> Result<Option<String>, ConfigError> {
    source
        .get(key)
        .map_err(|_| ConfigError::invalid(key, "must be valid UTF-8"))
}

fn required(source: &impl ConfigSource, key: &str) -> Result<String, ConfigError> {
    auth_value(source, key)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ConfigError::invalid(key, "a non-empty value is required"))
}

fn provider_name(source: &impl ConfigSource) -> Result<String, ConfigError> {
    let value = auth_value(source, "GAZEL_OIDC_PROVIDER_NAME")?
        .unwrap_or_else(|| String::from("OpenID Connect"));
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(ConfigError::invalid(
            "GAZEL_OIDC_PROVIDER_NAME",
            "must not be empty",
        ));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(ConfigError::invalid(
            "GAZEL_OIDC_PROVIDER_NAME",
            "must not contain control characters",
        ));
    }
    if trimmed.chars().count() > 80 {
        return Err(ConfigError::invalid(
            "GAZEL_OIDC_PROVIDER_NAME",
            "must contain at most 80 characters",
        ));
    }

    Ok(trimmed.to_string())
}

fn validate_auth_url(value: &str, key: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(value)
        .map_err(|_| ConfigError::invalid(key, "must be an absolute HTTP(S) URL"))?;

    if url.host().is_none() {
        return Err(ConfigError::invalid(key, "must include a host"));
    }
    if has_userinfo(value) || !url.username().is_empty() || url.password().is_some() {
        return Err(ConfigError::invalid(key, "must not include credentials"));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(ConfigError::invalid(
            key,
            "must not include a query string or fragment",
        ));
    }

    match url.scheme() {
        "https" => {}
        "http" if is_explicit_loopback(&url) => {}
        _ => {
            return Err(ConfigError::invalid(
                key,
                "must use HTTPS except on localhost, 127.0.0.1, or ::1",
            ));
        }
    }

    Ok(url)
}

fn has_userinfo(value: &str) -> bool {
    value.split_once("://").is_some_and(|(_, remainder)| {
        let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
        remainder[..authority_end].contains('@')
    })
}

fn is_explicit_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address == Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(address)) => address == Ipv6Addr::LOCALHOST,
        None => false,
    }
}

/// Parse a config value from `source`, falling back to `default` when the key
/// is absent or the value cannot be parsed.
fn parse_or<T: FromStr>(source: &impl ConfigSource, key: &str, default: T) -> T {
    source
        .get(key)
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct MockConfigSource(HashMap<String, String>);

    impl ConfigSource for MockConfigSource {
        fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError> {
            Ok(self.0.get(key).cloned())
        }
    }

    struct NonUnicodeConfigSource {
        inner: MockConfigSource,
        invalid_key: &'static str,
    }

    impl ConfigSource for NonUnicodeConfigSource {
        fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError> {
            if key == self.invalid_key {
                Err(std::env::VarError::NotUnicode(std::ffi::OsString::from(
                    "invalid",
                )))
            } else {
                self.inner.get(key)
            }
        }
    }

    fn mock(entries: &[(&str, &str)]) -> MockConfigSource {
        MockConfigSource(
            entries
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
        )
    }

    fn enabled_source() -> MockConfigSource {
        mock(&[
            ("GAZEL_AUTH_ENABLED", "true"),
            ("GAZEL_EXTERNAL_URL", "https://gazel.example"),
            ("GAZEL_OIDC_ISSUER", "https://id.example/realms/gazel"),
            ("GAZEL_OIDC_CLIENT_ID", "gazel"),
            ("GAZEL_OIDC_CLIENT_SECRET", "secret"),
        ])
    }

    fn enabled_entries() -> HashMap<String, String> {
        enabled_source().0
    }

    #[test]
    fn defaults_when_no_env_vars() {
        let config = Config::load_from(&mock(&[])).expect("defaults should be valid");
        assert_eq!(config.port, 4110);
        assert_eq!(config.db_path, "/data/gazel.db");
        assert_eq!(config.log_level, "info");
        assert!(config.auth.is_none());
    }

    #[test]
    fn explicit_false_ignores_auth_only_values() {
        let config = Config::load_from(&mock(&[
            ("GAZEL_AUTH_ENABLED", "false"),
            ("GAZEL_EXTERNAL_URL", "not a URL"),
            ("GAZEL_OIDC_PROVIDER_NAME", ""),
        ]))
        .expect("disabled auth values should be ignored");

        assert!(config.auth.is_none());
    }

    #[test]
    fn malformed_enable_flags_are_rejected() {
        for value in ["", "1", "TRUE", "yes", " false "] {
            let error = Config::load_from(&mock(&[("GAZEL_AUTH_ENABLED", value)]))
                .expect_err("malformed flag should fail");
            assert!(error.to_string().contains("GAZEL_AUTH_ENABLED"));
        }
    }

    #[test]
    fn non_unicode_authentication_values_fail_closed() {
        let invalid_flag = NonUnicodeConfigSource {
            inner: mock(&[]),
            invalid_key: "GAZEL_AUTH_ENABLED",
        };
        let error = Config::load_from(&invalid_flag).expect_err("invalid flag should fail");
        assert!(error.to_string().contains("GAZEL_AUTH_ENABLED"));

        for key in [
            "GAZEL_EXTERNAL_URL",
            "GAZEL_OIDC_ISSUER",
            "GAZEL_OIDC_CLIENT_ID",
            "GAZEL_OIDC_CLIENT_SECRET",
            "GAZEL_OIDC_PROVIDER_NAME",
        ] {
            let source = NonUnicodeConfigSource {
                inner: enabled_source(),
                invalid_key: key,
            };
            let error = Config::load_from(&source).expect_err("invalid auth value should fail");
            assert!(error.to_string().contains(key));
        }
    }

    #[test]
    fn explicit_false_ignores_non_unicode_auth_only_values() {
        let source = NonUnicodeConfigSource {
            inner: mock(&[("GAZEL_AUTH_ENABLED", "false")]),
            invalid_key: "GAZEL_OIDC_PROVIDER_NAME",
        };
        let config = Config::load_from(&source).expect("disabled auth value should be ignored");
        assert!(config.auth.is_none());
    }

    #[test]
    fn enabled_auth_requires_every_non_empty_value() {
        for key in [
            "GAZEL_EXTERNAL_URL",
            "GAZEL_OIDC_ISSUER",
            "GAZEL_OIDC_CLIENT_ID",
            "GAZEL_OIDC_CLIENT_SECRET",
        ] {
            let mut missing = enabled_entries();
            missing.remove(key);
            let error = Config::load_from(&MockConfigSource(missing))
                .expect_err("missing value should fail");
            assert!(error.to_string().contains(key));

            for empty in ["", "   "] {
                let mut entries = enabled_entries();
                entries.insert(key.to_string(), empty.to_string());
                let error = Config::load_from(&MockConfigSource(entries))
                    .expect_err("empty value should fail");
                assert!(error.to_string().contains(key));
            }
        }
    }

    #[test]
    fn enabled_auth_accepts_https_and_explicit_loopback_http_urls() {
        for external_url in [
            "https://gazel.example",
            "http://localhost:4110",
            "http://127.0.0.1:4110",
            "http://[::1]:4110",
        ] {
            let mut entries = enabled_entries();
            entries.insert("GAZEL_EXTERNAL_URL".to_string(), external_url.to_string());
            Config::load_from(&MockConfigSource(entries)).expect("external URL should be valid");
        }

        for issuer in [
            "https://id.example/realms/gazel",
            "http://localhost:8080/issuer",
            "http://127.0.0.1:8080/issuer",
            "http://[::1]:8080/issuer",
        ] {
            let mut entries = enabled_entries();
            entries.insert("GAZEL_OIDC_ISSUER".to_string(), issuer.to_string());
            Config::load_from(&MockConfigSource(entries)).expect("issuer URL should be valid");
        }
    }

    #[test]
    fn enabled_auth_rejects_unsafe_or_ambiguous_urls() {
        for key in ["GAZEL_EXTERNAL_URL", "GAZEL_OIDC_ISSUER"] {
            for value in [
                "relative/path",
                "ftp://example.com",
                "http://example.com",
                "https://user@example.com",
                "https://example.com?query=value",
                "https://example.com#fragment",
            ] {
                let mut entries = enabled_entries();
                entries.insert(key.to_string(), value.to_string());
                let error = Config::load_from(&MockConfigSource(entries))
                    .expect_err("unsafe URL should fail");
                assert!(error.to_string().contains(key));
            }
        }
    }

    #[test]
    fn external_url_must_be_root_mounted() {
        let mut entries = enabled_entries();
        entries.insert(
            "GAZEL_EXTERNAL_URL".to_string(),
            "https://gazel.example/subpath".to_string(),
        );

        let error = Config::load_from(&MockConfigSource(entries))
            .expect_err("non-root external URL should fail");
        assert!(error.to_string().contains("GAZEL_EXTERNAL_URL"));
    }

    #[test]
    fn callback_url_uses_only_the_normalized_external_origin() {
        let mut entries = enabled_entries();
        entries.insert(
            "GAZEL_EXTERNAL_URL".to_string(),
            "https://gazel.example:8443/".to_string(),
        );

        let config = Config::load_from(&MockConfigSource(entries)).expect("config should load");
        let auth = config.auth.expect("auth should be enabled");
        assert_eq!(auth.external_url.as_str(), "https://gazel.example:8443/");
        assert_eq!(
            auth.callback_url.as_str(),
            "https://gazel.example:8443/auth/callback"
        );
    }

    #[test]
    fn provider_name_defaults_and_custom_value_is_trimmed() {
        let default = Config::load_from(&enabled_source()).expect("config should load");
        assert_eq!(
            default.auth.expect("auth should be enabled").provider_name,
            "OpenID Connect"
        );

        let mut entries = enabled_entries();
        entries.insert(
            "GAZEL_OIDC_PROVIDER_NAME".to_string(),
            "  Authentik  ".to_string(),
        );
        let custom = Config::load_from(&MockConfigSource(entries)).expect("config should load");
        assert_eq!(
            custom.auth.expect("auth should be enabled").provider_name,
            "Authentik"
        );
    }

    #[test]
    fn invalid_provider_names_are_rejected() {
        for name in [String::new(), "   ".to_string(), "bad\nname".to_string()] {
            let mut entries = enabled_entries();
            entries.insert("GAZEL_OIDC_PROVIDER_NAME".to_string(), name);
            let error = Config::load_from(&MockConfigSource(entries))
                .expect_err("invalid provider name should fail");
            assert!(error.to_string().contains("GAZEL_OIDC_PROVIDER_NAME"));
        }

        let mut entries = enabled_entries();
        entries.insert("GAZEL_OIDC_PROVIDER_NAME".to_string(), "x".repeat(81));
        let error = Config::load_from(&MockConfigSource(entries))
            .expect_err("overlength provider name should fail");
        assert!(error.to_string().contains("GAZEL_OIDC_PROVIDER_NAME"));
    }

    #[test]
    fn legacy_values_keep_their_existing_fallback_behavior() {
        let config = Config::load_from(&mock(&[
            ("GAZEL_PORT", "not_a_number"),
            ("GAZEL_DB_PATH", "/custom/path.db"),
            ("GAZEL_LOG_LEVEL", "trace"),
        ]))
        .expect("legacy config should load");
        assert_eq!(config.port, 4110);
        assert_eq!(config.db_path, "/custom/path.db");
        assert_eq!(config.log_level, "trace");
    }
}
