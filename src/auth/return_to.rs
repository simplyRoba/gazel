pub(crate) const RETURN_TO_MAX_BYTES: usize = 2_048;

/// A validated serialized local document target, retained without normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SafeReturnTo(String);

impl SafeReturnTo {
    pub(super) fn new(value: Option<&str>) -> Self {
        Self(validate_serialized_target(value))
    }

    pub(super) fn fallback() -> Self {
        Self(String::from("/"))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn validate_return_to(value: Option<&str>) -> String {
    SafeReturnTo::new(value).0
}

fn validate_serialized_target(value: Option<&str>) -> String {
    let Some(value) = value else {
        return String::from("/");
    };
    if value.len() > RETURN_TO_MAX_BYTES || !value.starts_with('/') || value.starts_with("//") {
        return String::from("/");
    }

    let Some(decoded) = decode_percent_encoded(value, false) else {
        return String::from("/");
    };
    if decoded.contains('\\') || decoded.chars().any(char::is_control) {
        return String::from("/");
    }

    let path_end = value.find(['?', '#']).unwrap_or(value.len());
    let path = &value[..path_end];
    let Some(decoded_path) = decode_percent_encoded(path, false) else {
        return String::from("/");
    };
    if decoded_path.contains(['%', '?', '#'])
        || decoded_path.matches('/').count() != path.matches('/').count()
        || is_reserved_path(&decoded_path)
        || has_dot_segment(&decoded_path)
        || url::Url::parse(&format!("https://auth.invalid{value}")).is_err()
    {
        return String::from("/");
    }
    value.to_string()
}

fn has_dot_segment(path: &str) -> bool {
    path.split('/')
        .any(|segment| segment == "." || segment == "..")
}

fn is_reserved_path(path: &str) -> bool {
    ["/api", "/auth", "/health", "/login"]
        .iter()
        .chain(super::app::NON_DOCUMENT_ROOTS)
        .any(|reserved| path == *reserved || path.starts_with(&format!("{reserved}/")))
}

pub(super) fn return_to_from_query(raw_query: Option<&str>) -> String {
    let Some(raw_query) = raw_query else {
        return SafeReturnTo::fallback().0;
    };
    for pair in raw_query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if decode_query_component(key).as_deref() == Some("return_to") {
            return validate_return_to(decode_query_component(value).as_deref());
        }
    }
    String::from("/")
}

pub(super) fn decode_query_component(value: &str) -> Option<String> {
    decode_percent_encoded(value, true)
}

fn decode_percent_encoded(value: &str, plus_as_space: bool) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' if plus_as_space => decoded.push(b' '),
            b'%' => {
                let high = *bytes.get(index + 1)?;
                let low = *bytes.get(index + 2)?;
                decoded.push(hex(high)? << 4 | hex(low)?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_return_vectors_follow_the_common_contract() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/auth/return-targets.json"
        ))
        .expect("return fixture should parse");
        for vector in fixture["vectors"].as_array().unwrap() {
            let input = &vector["input"];
            let value = if input.is_null() {
                None
            } else if let Some(value) = input.as_str() {
                Some(value.to_owned())
            } else {
                Some(format!(
                    "{}{}{}",
                    input["prefix"].as_str().unwrap(),
                    input["repeat"]
                        .as_str()
                        .unwrap()
                        .repeat(input["count"].as_u64().unwrap().try_into().unwrap()),
                    input["suffix"].as_str().unwrap()
                ))
            };
            let expected = if vector["target"][super::super::app::FIXTURE_APP]
                .as_bool()
                .unwrap()
            {
                value.as_deref().unwrap_or("/")
            } else {
                "/"
            };
            assert_eq!(
                validate_return_to(value.as_deref()),
                expected,
                "{}",
                vector["id"]
            );

            // Exercise exactly one form decoding, including wire size > serialized size.
            let wire = value.as_ref().map(|value| {
                url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("return_to", value)
                    .finish()
            });
            assert_eq!(
                return_to_from_query(wire.as_deref()),
                expected,
                "{}",
                vector["id"]
            );
        }
    }

    #[test]
    fn return_target_validation_rejects_ambiguous_or_reserved_locations() {
        assert_eq!(
            validate_return_to(Some("/settings?tab=data#export")),
            "/settings?tab=data#export"
        );
        let encoded_boundary = format!("/{}", "%61".repeat(RETURN_TO_MAX_BYTES - 1));
        assert_eq!(validate_return_to(Some(&encoded_boundary)), "/");

        for unsafe_target in [
            "https://attacker.example/",
            "//attacker.example/",
            "%2Fsettings",
            "/x/../auth/login",
            "/x/%2e%2e/auth/login",
            "/./settings",
            "/\\attacker",
            "/foo%5Cbar",
            "/settings%0Aheader",
            "/%61pi/vehicles",
            "/api",
            "/auth/login",
            "/health",
            "/login",
        ] {
            assert_eq!(validate_return_to(Some(unsafe_target)), "/");
        }
        assert_eq!(
            validate_return_to(Some(&format!("/{}", "%61".repeat(RETURN_TO_MAX_BYTES)))),
            "/"
        );
        assert_eq!(validate_return_to(None), "/");
    }
}
