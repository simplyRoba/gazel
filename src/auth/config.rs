use std::env::VarError;
use std::fmt;

use url::Url;

use super::app;

/// Fallible source for authentication settings, using unprefixed setting keys.
pub trait AuthConfigSource {
    /// # Errors
    /// Returns an error when a present value cannot be read as Unicode.
    fn get(&self, key: &str) -> Result<Option<String>, VarError>;
}

/// Validated configuration for optional built-in OIDC authentication.
#[derive(Clone)]
pub struct OidcConfig {
    /// Normalized public origin used for browser redirects.
    pub external_url: Url,
    pub callback_url: Url,
    /// Exact validated OIDC issuer identifier used for discovery.
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
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
        app::cookie_secure(self)
    }
}

/// Value-free authentication configuration failure categories.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthConfigError {
    InvalidEnabledFlag,
    NonUnicodeValue(String),
    MissingRequiredValue(String),
    InvalidExternalUrl,
    InvalidIssuerUrl,
    InvalidProviderName,
}

impl fmt::Display for AuthConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (key, reason) = match self {
            Self::InvalidEnabledFlag => ("AUTH_ENABLED", "expected exactly `true` or `false`"),
            Self::NonUnicodeValue(key) => (key.as_str(), "must be valid UTF-8"),
            Self::MissingRequiredValue(key) => (key.as_str(), "a non-empty value is required"),
            Self::InvalidExternalUrl => ("EXTERNAL_URL", "must be a safe root-mounted origin"),
            Self::InvalidIssuerUrl => ("OIDC_ISSUER", "must be a safe absolute issuer URL"),
            Self::InvalidProviderName => (
                "OIDC_PROVIDER_NAME",
                "must be a trimmed non-empty label of at most 80 characters without controls",
            ),
        };
        write!(formatter, "invalid {}{key}: {reason}", app::ENV_PREFIX)
    }
}

impl std::error::Error for AuthConfigError {}

/// Load only authentication settings, leaving unrelated configuration untouched.
///
/// # Errors
/// Rejects malformed flags, non-Unicode values read by the enabled parser,
/// missing required settings, unsafe URLs, or an invalid provider label.
pub fn load_from(source: &impl AuthConfigSource) -> Result<Option<OidcConfig>, AuthConfigError> {
    match value(source, "AUTH_ENABLED")?.as_deref() {
        None | Some("false") => return Ok(None),
        Some("true") => {}
        Some(_) => return Err(AuthConfigError::InvalidEnabledFlag),
    }

    let external_value = required(source, "EXTERNAL_URL")?;
    let issuer = required(source, "OIDC_ISSUER")?;
    let client_id = required(source, "OIDC_CLIENT_ID")?;
    let client_secret = required(source, "OIDC_CLIENT_SECRET")?;
    let mut external_url =
        validate_config_url(&external_value).ok_or(AuthConfigError::InvalidExternalUrl)?;
    if external_url.path() != "/" {
        return Err(AuthConfigError::InvalidExternalUrl);
    }
    external_url.set_path("/");
    let callback_url = external_url
        .join("auth/callback")
        .map_err(|_| AuthConfigError::InvalidExternalUrl)?;
    validate_config_url(&issuer).ok_or(AuthConfigError::InvalidIssuerUrl)?;

    let provider_name =
        value(source, "OIDC_PROVIDER_NAME")?.unwrap_or_else(|| String::from("OpenID Connect"));
    let provider_name = provider_name.trim();
    if provider_name.is_empty()
        || provider_name.chars().any(char::is_control)
        || provider_name.chars().count() > 80
    {
        return Err(AuthConfigError::InvalidProviderName);
    }

    Ok(Some(OidcConfig {
        external_url,
        callback_url,
        issuer,
        client_id,
        client_secret,
        provider_name: provider_name.to_owned(),
    }))
}

fn value(source: &impl AuthConfigSource, key: &str) -> Result<Option<String>, AuthConfigError> {
    source
        .get(key)
        .map_err(|_| AuthConfigError::NonUnicodeValue(key.to_owned()))
}

fn required(source: &impl AuthConfigSource, key: &str) -> Result<String, AuthConfigError> {
    value(source, key)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AuthConfigError::MissingRequiredValue(key.to_owned()))
}

fn validate_config_url(value: &str) -> Option<Url> {
    // Configuration never uses the provider fixture exception, even in test builds.
    let url = app::config_policy().validate(value).ok()?;
    (url.query().is_none()).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LabelSource(Option<String>);

    impl AuthConfigSource for LabelSource {
        fn get(&self, key: &str) -> Result<Option<String>, VarError> {
            Ok(match key {
                "AUTH_ENABLED" => Some(String::from("true")),
                "EXTERNAL_URL" | "OIDC_ISSUER" => Some(String::from("https://auth.example")),
                "OIDC_CLIENT_ID" | "OIDC_CLIENT_SECRET" => Some(String::from("fixture")),
                "OIDC_PROVIDER_NAME" => self.0.clone(),
                _ => None,
            })
        }
    }

    fn label(value: &serde_json::Value) -> Option<String> {
        if value.is_null() {
            None
        } else if let Some(value) = value.as_str() {
            Some(value.to_owned())
        } else {
            Some(format!(
                "{}{}{}",
                value["prefix"].as_str().unwrap_or(""),
                value["repeat"]
                    .as_str()
                    .unwrap()
                    .repeat(value["count"].as_u64().unwrap().try_into().unwrap()),
                value["suffix"].as_str().unwrap_or("")
            ))
        }
    }

    #[test]
    fn shared_provider_labels_use_the_actual_config_loader() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/auth/provider-labels.json"
        ))
        .expect("provider label fixture should parse");
        for vector in fixture["vectors"].as_array().unwrap() {
            let result = load_from(&LabelSource(label(&vector["input"])));
            if let Some(expected) = label(&vector["normalized"]) {
                assert_eq!(
                    result.unwrap().unwrap().provider_name,
                    expected,
                    "{}",
                    vector["id"]
                );
            } else {
                assert_eq!(
                    result.unwrap_err(),
                    AuthConfigError::InvalidProviderName,
                    "{}",
                    vector["id"]
                );
            }
        }
    }
}
