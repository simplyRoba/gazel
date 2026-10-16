use std::str::FromStr;

use crate::auth::{app::AuthSource, config as auth_config};
pub use auth_config::{AuthConfigError as ConfigError, OidcConfig};

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
        let auth = auth_config::load_from(&AuthSource(source))?;

        Ok(Self {
            port: parse_or(source, "GAZEL_PORT", 4110),
            db_path: parse_or(source, "GAZEL_DB_PATH", String::from("/data/gazel.db")),
            log_level: parse_or(source, "GAZEL_LOG_LEVEL", String::from("info")),
            auth,
        })
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
                    "private-value-sentinel",
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
        assert!(!error.to_string().contains("private-value-sentinel"));

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
            assert!(!error.to_string().contains("private-value-sentinel"));
        }
    }

    #[test]
    fn disabled_auth_ignores_non_unicode_oidc_values() {
        for flag in [None, Some("false")] {
            for key in [
                "GAZEL_EXTERNAL_URL",
                "GAZEL_OIDC_ISSUER",
                "GAZEL_OIDC_CLIENT_ID",
                "GAZEL_OIDC_CLIENT_SECRET",
                "GAZEL_OIDC_PROVIDER_NAME",
            ] {
                let source = NonUnicodeConfigSource {
                    inner: flag
                        .map_or_else(|| mock(&[]), |flag| mock(&[("GAZEL_AUTH_ENABLED", flag)])),
                    invalid_key: key,
                };
                let config = Config::load_from(&source).expect("disabled OIDC value is ignored");
                assert!(config.auth.is_none(), "{key}");
            }
        }
    }

    #[test]
    fn enabled_auth_preserves_unrelated_configuration_fallbacks() {
        for key in ["GAZEL_PORT", "GAZEL_DB_PATH", "GAZEL_LOG_LEVEL"] {
            let source = NonUnicodeConfigSource {
                inner: enabled_source(),
                invalid_key: key,
            };
            let config = Config::load_from(&source).expect("ordinary settings stay lenient");
            assert!(config.auth.is_some());
            assert_eq!(config.port, 4110);
            assert_eq!(config.db_path, "/data/gazel.db");
            assert_eq!(config.log_level, "info");
        }
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
            let config = Config::load_from(&MockConfigSource(entries))
                .expect("external URL should be valid");
            assert_eq!(
                config.auth.unwrap().secure_cookie(),
                external_url.starts_with("https:"),
                "{external_url}"
            );
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
    fn oidc_issuer_preserves_the_validated_text() {
        for issuer in ["https://id.example", "https://id.example/"] {
            let mut entries = enabled_entries();
            entries.insert("GAZEL_OIDC_ISSUER".to_string(), issuer.to_string());

            let config = Config::load_from(&MockConfigSource(entries)).expect("config should load");

            assert_eq!(config.auth.expect("auth should be enabled").issuer, issuer);
        }
    }

    #[test]
    fn enabled_auth_rejects_unsafe_or_ambiguous_urls() {
        for key in ["GAZEL_EXTERNAL_URL", "GAZEL_OIDC_ISSUER"] {
            for value in [
                "relative/path",
                "ftp://example.com",
                "http://example.com",
                "http://localhost.example",
                "http://127.0.0.2",
                "http://[::2]",
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
    fn provider_name_uses_unicode_scalars_and_rejects_controls() {
        for (name, valid) in [
            ("é".repeat(80), true),
            ("é".repeat(81), false),
            ("🌱".repeat(80), true),
            ("🌱".repeat(81), false),
            (String::from("before\u{0085}after"), false),
            (String::from("before\u{009f}after"), false),
            (String::from("before\0after"), false),
        ] {
            let mut entries = enabled_entries();
            entries.insert("GAZEL_OIDC_PROVIDER_NAME".to_string(), name.clone());
            assert_eq!(
                Config::load_from(&MockConfigSource(entries)).is_ok(),
                valid,
                "{name:?}"
            );
        }
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
