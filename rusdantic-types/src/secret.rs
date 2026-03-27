//! Secret types with redacted Debug/Display output.
//!
//! These types wrap sensitive values and ensure they never appear in logs,
//! debug output, or error messages.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Constant-time byte comparison to prevent timing side-channel attacks.
/// Always compares all bytes regardless of where the first difference occurs.
/// Returns `false` immediately only if lengths differ (length is not secret).
#[inline]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// A secret string value that is redacted in Debug and Display output.
///
/// The inner value is accessible via `expose_secret()` but never appears
/// in formatted output, preventing accidental PII leaks in logs.
///
/// # Example
/// ```
/// use rusdantic_types::SecretStr;
/// let s = SecretStr::new("my-api-key");
/// assert_eq!(format!("{:?}", s), "SecretStr(\"**********\")");
/// assert_eq!(s.expose_secret(), "my-api-key");
/// ```
#[derive(Clone)]
pub struct SecretStr(String);

impl SecretStr {
    /// Create a new secret string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Expose the secret value. Use sparingly — this is the only way
    /// to access the inner string.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretStr(\"**********\")")
    }
}

impl fmt::Display for SecretStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**********")
    }
}

impl PartialEq for SecretStr {
    /// Constant-time comparison to prevent timing side-channel attacks.
    /// This is critical because SecretStr holds sensitive values (API keys,
    /// passwords, tokens) and standard string comparison short-circuits
    /// on the first differing byte.
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(self.0.as_bytes(), other.0.as_bytes())
    }
}

impl Eq for SecretStr {}

impl Serialize for SecretStr {
    /// Serialization skips the secret value entirely, emitting null.
    ///
    /// This prevents accidental leakage of secrets into databases, logs,
    /// or API responses. To intentionally serialize the secret value,
    /// use `expose_secret()` and serialize it manually.
    ///
    /// To serialize as a redacted placeholder string, use the
    /// `serialize_redacted()` method instead.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl SecretStr {
    /// Serialize the value as a redacted placeholder "**********".
    /// Use this when you want to indicate a secret exists without exposing it.
    pub fn serialize_redacted<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        "**********".serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SecretStr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

/// A secret bytes value that is redacted in Debug and Display output.
#[derive(Clone)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Create new secret bytes.
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }

    /// Expose the secret bytes.
    pub fn expose_secret(&self) -> &[u8] {
        &self.0
    }

    /// Consume and return inner bytes.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBytes(\"**********\")")
    }
}

impl fmt::Display for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**********")
    }
}

impl PartialEq for SecretBytes {
    /// Constant-time comparison to prevent timing side-channel attacks.
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(&self.0, &other.0)
    }
}

impl Eq for SecretBytes {}

impl Serialize for SecretBytes {
    /// Serializes as null to prevent accidental secret leakage.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for SecretBytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s.into_bytes()))
    }
}

/// A generic secret wrapper for any type.
///
/// Wraps any value and redacts it in Debug/Display output.
/// The inner value is only accessible via `expose_secret()`.
#[derive(Clone)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Create a new secret wrapper.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Expose the secret value.
    pub fn expose_secret(&self) -> &T {
        &self.0
    }

    /// Consume and return the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret(\"**********\")")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**********")
    }
}

impl<T: PartialEq> PartialEq for Secret<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Secret<T> {}

impl<T: Serialize> Serialize for Secret<T> {
    /// Serializes as null to prevent accidental secret leakage.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Secret<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_str_debug_redacted() {
        let s = SecretStr::new("my-api-key");
        assert_eq!(format!("{:?}", s), "SecretStr(\"**********\")");
    }

    #[test]
    fn test_secret_str_display_redacted() {
        let s = SecretStr::new("my-api-key");
        assert_eq!(format!("{}", s), "**********");
    }

    #[test]
    fn test_secret_str_expose() {
        let s = SecretStr::new("my-api-key");
        assert_eq!(s.expose_secret(), "my-api-key");
    }

    #[test]
    fn test_secret_str_serialize_null() {
        // SecretStr serializes as null to prevent accidental data leakage.
        // Use expose_secret() for intentional serialization.
        let s = SecretStr::new("my-api-key");
        let json = serde_json::to_value(&s).unwrap();
        assert!(json.is_null());
    }

    #[test]
    fn test_secret_str_deserialize() {
        let s: SecretStr = serde_json::from_value(serde_json::json!("my-api-key")).unwrap();
        assert_eq!(s.expose_secret(), "my-api-key");
    }

    #[test]
    fn test_secret_generic() {
        let s = Secret::new(42i32);
        assert_eq!(*s.expose_secret(), 42);
        assert_eq!(format!("{:?}", s), "Secret(\"**********\")");
    }

    #[test]
    fn test_secret_bytes_debug() {
        let s = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(format!("{:?}", s), "SecretBytes(\"**********\")");
    }
}
