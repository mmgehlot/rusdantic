//! Network types: validated URLs, IP addresses.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// A validated HTTP/HTTPS URL.
///
/// # Example
/// ```
/// use rusdantic_types::HttpUrl;
/// let u = HttpUrl::new("https://example.com").unwrap();
/// assert!(HttpUrl::new("ftp://files.com").is_err()); // not HTTP
/// assert!(HttpUrl::new("not-a-url").is_err());
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct HttpUrl(String);

impl HttpUrl {
    /// Create a new validated HTTP URL.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let s = value.into();
        match url::Url::parse(&s) {
            Ok(parsed) => {
                if parsed.scheme() != "http" && parsed.scheme() != "https" {
                    return Err(format!(
                        "URL must use http or https scheme, got '{}'",
                        parsed.scheme()
                    ));
                }
                if parsed.host().is_none() {
                    return Err("URL must have a host".to_string());
                }
                Ok(Self(s))
            }
            Err(e) => Err(format!("invalid URL: {}", e)),
        }
    }

    /// Get the inner URL string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for HttpUrl {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpUrl({:?})", self.0)
    }
}

impl fmt::Display for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for HttpUrl {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HttpUrl {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_url_valid() {
        assert!(HttpUrl::new("https://example.com").is_ok());
        assert!(HttpUrl::new("http://example.com/path").is_ok());
        assert!(HttpUrl::new("https://sub.domain.example.com:8080/path?q=1").is_ok());
    }

    #[test]
    fn test_http_url_invalid_scheme() {
        assert!(HttpUrl::new("ftp://files.example.com").is_err());
    }

    #[test]
    fn test_http_url_invalid() {
        assert!(HttpUrl::new("not-a-url").is_err());
        assert!(HttpUrl::new("").is_err());
    }

    #[test]
    fn test_http_url_serialize() {
        let u = HttpUrl::new("https://example.com").unwrap();
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json, "https://example.com");
    }

    #[test]
    fn test_http_url_deserialize() {
        let u: HttpUrl = serde_json::from_value(serde_json::json!("https://example.com")).unwrap();
        assert_eq!(u.as_str(), "https://example.com");
    }

    #[test]
    fn test_http_url_deserialize_invalid() {
        let result: Result<HttpUrl, _> = serde_json::from_value(serde_json::json!("not-a-url"));
        assert!(result.is_err());
    }
}
