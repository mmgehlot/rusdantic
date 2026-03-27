//! Email format validation rule.
//!
//! Validates that a string value conforms to a reasonable email format.
//! Uses a regex pattern based on the HTML5 email specification, which is
//! a practical subset of RFC 5322. This balances correctness with usability —
//! most "technically valid" but practically useless email addresses are rejected.

use crate::error::{PathSegment, ValidationError, ValidationErrors};
use crate::rules::AsStr;
use std::sync::OnceLock;

/// The email validation regex.
///
/// Based on the HTML5 spec's email pattern, which accepts addresses like:
/// - `user@example.com`
/// - `first.last@sub.domain.org`
/// - `user+tag@example.com`
///
/// And rejects obviously invalid patterns like:
/// - `@example.com` (no local part)
/// - `user@` (no domain)
/// - `user@.com` (domain starting with dot)
/// - `user@@example.com` (double @)
///
/// Note: This is intentionally not a full RFC 5322 parser. The RFC allows
/// many formats that are technically valid but never used in practice
/// (quoted strings, comments, IP address literals, etc.).
static EMAIL_REGEX: OnceLock<regex::Regex> = OnceLock::new();

/// The email regex pattern string. Shared with rusdantic-types::EmailStr.
pub const EMAIL_REGEX_PATTERN: &str =
    r"(?i)^[a-z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*\.[a-z]{2,}$";

/// Get or compile the email validation regex (compiled once, reused forever).
pub fn get_email_regex() -> &'static regex::Regex {
    EMAIL_REGEX.get_or_init(|| {
        // HTML5-spec inspired email regex:
        // Local part: one or more characters from [a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]
        // @ separator
        // Domain: one or more labels separated by dots, each label is alphanumeric
        //         with hyphens allowed (but not at start/end)
        regex::Regex::new(
            r"(?i)^[a-z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*\.[a-z]{2,}$"
        ).expect("email regex is valid")
    })
}

/// Validate that the value is a valid email address.
///
/// Uses a practical email regex based on the HTML5 specification.
/// This is more permissive than some strict validators but rejects
/// obviously invalid formats.
pub fn validate_email<T: AsStr>(
    value: &T,
    path: &[PathSegment],
    errors: &mut ValidationErrors,
) {
    let s = value.as_str_ref();

    // Quick checks before regex (performance optimization)
    if s.is_empty() || !s.contains('@') || s.len() > 254 {
        errors.add(
            ValidationError::new("email", "invalid email format")
                .with_path(path.to_vec()),
        );
        return;
    }

    if !get_email_regex().is_match(s) {
        errors.add(
            ValidationError::new("email", "invalid email format")
                .with_path(path.to_vec()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Vec<PathSegment> {
        vec![PathSegment::Field(name.to_string())]
    }

    #[test]
    fn test_valid_emails() {
        let valid = vec![
            "user@example.com",
            "first.last@example.com",
            "user+tag@example.com",
            "user@sub.domain.example.com",
            "user123@example.com",
            "USER@EXAMPLE.COM",
            "user@example.co.uk",
            "user@123.123.123.example.com",
            "test.email-with-dash@example.com",
        ];
        for email in valid {
            let mut errors = ValidationErrors::new();
            validate_email(&email.to_string(), &path("email"), &mut errors);
            assert!(errors.is_empty(), "Expected valid: {}", email);
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid = vec![
            "",                     // empty
            "not-an-email",         // no @
            "@example.com",         // no local part
            "user@",                // no domain
            "user@@example.com",    // double @
            "user@.com",            // domain starts with dot
            "user@com",             // single-label domain (no TLD)
            " user@example.com",    // leading space
            "user@example.com ",    // trailing space
            "user@-example.com",    // domain starts with hyphen
        ];
        for email in invalid {
            let mut errors = ValidationErrors::new();
            validate_email(&email.to_string(), &path("email"), &mut errors);
            assert!(!errors.is_empty(), "Expected invalid: '{}'", email);
            assert_eq!(errors.errors()[0].code, "email");
        }
    }

    #[test]
    fn test_email_too_long() {
        // RFC 5321 limits email to 254 characters total
        let long_local = "a".repeat(250);
        let email = format!("{}@example.com", long_local);
        assert!(email.len() > 254);
        let mut errors = ValidationErrors::new();
        validate_email(&email, &path("email"), &mut errors);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_email_error_code() {
        let mut errors = ValidationErrors::new();
        validate_email(&"invalid".to_string(), &path("email"), &mut errors);
        assert_eq!(errors.errors()[0].code, "email");
        assert_eq!(errors.errors()[0].message, "invalid email format");
    }
}
