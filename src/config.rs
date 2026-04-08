use std::str::FromStr;

/// Abstraction over configuration sources, enabling testable config loading
/// without touching real environment variables.
pub trait ConfigSource {
    /// Returns the value for the given key, or `None` if not set.
    fn get(&self, key: &str) -> Option<String>;
}

/// Reads configuration from environment variables.
pub struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Application configuration read from environment variables.
pub struct Config {
    /// HTTP server listen port.
    pub port: u16,
    /// Filesystem path to the `SQLite` database.
    pub db_path: String,
    /// Tracing log-level filter.
    pub log_level: String,
}

impl Config {
    /// Load configuration from real environment variables.
    pub fn load() -> Self {
        Self::load_from(&EnvConfigSource)
    }

    /// Load configuration from an arbitrary source.
    ///
    /// # Errors
    ///
    /// This function does not error — invalid or missing values fall back to
    /// defaults.
    pub fn load_from(source: &impl ConfigSource) -> Self {
        Self {
            port: parse_or(source, "GAZEL_PORT", 4110),
            db_path: parse_or(source, "GAZEL_DB_PATH", String::from("/data/gazel.db")),
            log_level: parse_or(source, "GAZEL_LOG_LEVEL", String::from("info")),
        }
    }
}

/// Parse a config value from `source`, falling back to `default` when the key
/// is absent or the value cannot be parsed.
fn parse_or<T: FromStr>(source: &impl ConfigSource, key: &str, default: T) -> T {
    source
        .get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct MockConfigSource(HashMap<String, String>);

    impl ConfigSource for MockConfigSource {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
    }

    fn mock(entries: &[(&str, &str)]) -> MockConfigSource {
        MockConfigSource(
            entries
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        )
    }

    #[test]
    fn defaults_when_no_env_vars() {
        let config = Config::load_from(&mock(&[]));
        assert_eq!(config.port, 4110);
        assert_eq!(config.db_path, "/data/gazel.db");
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn custom_port() {
        let config = Config::load_from(&mock(&[("GAZEL_PORT", "9000")]));
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn custom_db_path() {
        let config = Config::load_from(&mock(&[("GAZEL_DB_PATH", "/tmp/test.db")]));
        assert_eq!(config.db_path, "/tmp/test.db");
    }

    #[test]
    fn custom_log_level() {
        let config = Config::load_from(&mock(&[("GAZEL_LOG_LEVEL", "debug")]));
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn invalid_port_falls_back_to_default() {
        let config = Config::load_from(&mock(&[("GAZEL_PORT", "not_a_number")]));
        assert_eq!(config.port, 4110);
    }

    #[test]
    fn all_custom_values() {
        let config = Config::load_from(&mock(&[
            ("GAZEL_PORT", "8080"),
            ("GAZEL_DB_PATH", "/custom/path.db"),
            ("GAZEL_LOG_LEVEL", "trace"),
        ]));
        assert_eq!(config.port, 8080);
        assert_eq!(config.db_path, "/custom/path.db");
        assert_eq!(config.log_level, "trace");
    }
}
