//! URL format validation rule.
//!
//! Validates that a string value is a valid URL using the `url` crate,
//! which implements the WHATWG URL Standard.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use crate::rules::AsStr;

/// Validate that the value is a valid URL.
///
/// Uses the `url` crate which implements the WHATWG URL Standard.
/// Accepts HTTP, HTTPS, FTP, and other standard schemes.
///
/// # Examples of valid URLs:
/// - `https://example.com`
/// - `https://example.com/path?query=value#fragment`
/// - `ftp://files.example.com/pub/`
///
/// # Examples of invalid URLs:
/// - `not-a-url`
/// - `://missing-scheme.com`
/// - `http://` (no host)
#[cfg(feature = "url-validation")]
pub fn validate_url<T: AsStr>(value: &T, path: &[PathSegment], errors: &mut ValidationErrors) {
    let s = value.as_str_ref();

    match url::Url::parse(s) {
        Ok(parsed) => {
            // Ensure the URL has a host (scheme-only URLs like "data:..." are
            // technically valid but not what most users expect)
            if parsed.host().is_none() && !matches!(parsed.scheme(), "data" | "mailto" | "tel") {
                errors.add(
                    ValidationError::new("url", "URL must have a host").with_path(path.to_vec()),
                );
            }
        }
        Err(_) => {
            errors.add(ValidationError::new("url", "invalid URL format").with_path(path.to_vec()));
        }
    }
}

/// Fallback URL validation when the `url` feature is not enabled.
/// Uses a simple heuristic check instead of full URL parsing.
#[cfg(not(feature = "url-validation"))]
pub fn validate_url<T: AsStr>(value: &T, path: &[PathSegment], errors: &mut ValidationErrors) {
    let s = value.as_str_ref();
    // Only accept http:// and https:// schemes
    if !s.starts_with("http://") && !s.starts_with("https://") {
        errors.add(
            ValidationError::new("url", "URL must use http or https scheme")
                .with_path(path.to_vec()),
        );
        return;
    }
    // Basic heuristic: must contain "://" and have something before and after it
    if !s.contains("://") || s.starts_with("://") || s.ends_with("://") {
        errors.add(ValidationError::new("url", "invalid URL format").with_path(path.to_vec()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Vec<PathSegment> {
        vec![PathSegment::Field(name.to_string())]
    }

    #[test]
    fn test_valid_urls() {
        let valid = vec![
            "https://example.com",
            "http://example.com/path",
            "https://sub.domain.example.com/path?q=1#frag",
            "https://example.com:8080",
            "ftp://files.example.com/pub/",
            "https://user:pass@example.com/",
            "https://example.com/path%20with%20spaces",
        ];
        for url_str in valid {
            let mut errors = ValidationErrors::new();
            validate_url(&url_str.to_string(), &path("url"), &mut errors);
            assert!(errors.is_empty(), "Expected valid: {}", url_str);
        }
    }

    #[test]
    fn test_invalid_urls() {
        let invalid = vec!["", "not a url", "just-text", "://missing-scheme.com"];
        for url_str in invalid {
            let mut errors = ValidationErrors::new();
            validate_url(&url_str.to_string(), &path("url"), &mut errors);
            assert!(!errors.is_empty(), "Expected invalid: '{}'", url_str);
        }
    }

    #[test]
    fn test_url_error_code() {
        let mut errors = ValidationErrors::new();
        validate_url(&"not-a-url".to_string(), &path("website"), &mut errors);
        assert_eq!(errors.errors()[0].code, "url");
    }
}
