//! Constrained string types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use std::sync::OnceLock;

/// A non-empty string (length >= 1).
///
/// Rejects empty strings at construction and deserialization time.
///
/// # Example
/// ```
/// use rusdantic_types::NonEmptyString;
/// let s = NonEmptyString::new("hello").unwrap();
/// assert_eq!(s.as_str(), "hello");
/// assert!(NonEmptyString::new("").is_err());
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Create a new non-empty string.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let s = value.into();
        if s.is_empty() {
            Err("string must not be empty".to_string())
        } else {
            Ok(Self(s))
        }
    }

    /// Get the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for NonEmptyString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NonEmptyString({:?})", self.0)
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// A validated email string.
///
/// Validates the email format at construction and deserialization time
/// using the same regex as the `email` validator.
///
/// # Example
/// ```
/// use rusdantic_types::EmailStr;
/// let e = EmailStr::new("user@example.com").unwrap();
/// assert!(EmailStr::new("invalid").is_err());
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmailStr(String);

/// Email regex — compiled once, reused forever.
static EMAIL_REGEX: OnceLock<regex::Regex> = OnceLock::new();

fn get_email_regex() -> &'static regex::Regex {
    EMAIL_REGEX.get_or_init(|| {
        regex::Regex::new(
            r"(?i)^[a-z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*\.[a-z]{2,}$"
        ).expect("email regex is valid")
    })
}

impl EmailStr {
    /// Create a new validated email string.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let s = value.into();
        if s.is_empty() || !s.contains('@') || s.len() > 254 {
            return Err("invalid email format".to_string());
        }
        if !get_email_regex().is_match(&s) {
            return Err("invalid email format".to_string());
        }
        Ok(Self(s))
    }

    /// Get the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for EmailStr {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for EmailStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmailStr({:?})", self.0)
    }
}

impl fmt::Display for EmailStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for EmailStr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EmailStr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_string_valid() {
        assert!(NonEmptyString::new("hello").is_ok());
        assert!(NonEmptyString::new("a").is_ok());
    }

    #[test]
    fn test_non_empty_string_invalid() {
        assert!(NonEmptyString::new("").is_err());
    }

    #[test]
    fn test_non_empty_string_deref() {
        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(&*s, "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_non_empty_string_serialize() {
        let s = NonEmptyString::new("test").unwrap();
        let json = serde_json::to_value(&s).unwrap();
        assert_eq!(json, "test");
    }

    #[test]
    fn test_non_empty_string_deserialize() {
        let s: NonEmptyString = serde_json::from_value(serde_json::json!("hello")).unwrap();
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_non_empty_string_deserialize_empty() {
        let result: Result<NonEmptyString, _> = serde_json::from_value(serde_json::json!(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_email_str_valid() {
        assert!(EmailStr::new("user@example.com").is_ok());
        assert!(EmailStr::new("first.last@sub.domain.com").is_ok());
    }

    #[test]
    fn test_email_str_invalid() {
        assert!(EmailStr::new("").is_err());
        assert!(EmailStr::new("not-email").is_err());
        assert!(EmailStr::new("@example.com").is_err());
    }

    #[test]
    fn test_email_str_serialize() {
        let e = EmailStr::new("user@example.com").unwrap();
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json, "user@example.com");
    }

    #[test]
    fn test_email_str_deserialize() {
        let e: EmailStr = serde_json::from_value(serde_json::json!("user@example.com")).unwrap();
        assert_eq!(e.as_str(), "user@example.com");
    }
}
