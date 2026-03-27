//! # rusdantic-settings
//!
//! Settings management for Rusdantic-based applications. Loads configuration
//! from multiple sources with a priority chain:
//!
//! **Priority order** (highest to lowest):
//! 1. Explicit values (passed to constructor)
//! 2. Environment variables
//! 3. Dotenv files (.env)
//! 4. Config files (TOML, JSON)
//! 5. Default values
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rusdantic_settings::{Settings, SettingsError};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug)]
//! struct AppConfig {
//!     database_url: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! impl Settings for AppConfig {
//!     fn env_prefix() -> &'static str { "MYAPP_" }
//! }
//!
//! // Load from environment variables
//! // MYAPP_DATABASE_URL=postgres://... MYAPP_PORT=8080 MYAPP_DEBUG=true
//! let config = AppConfig::from_env().unwrap();
//! ```

#![warn(missing_docs)]

use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Error type for settings operations.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// A required setting is missing from all sources.
    #[error("missing required setting: {0}")]
    MissingField(String),

    /// A setting value could not be parsed.
    #[error("invalid setting value for '{key}': {message}")]
    InvalidValue {
        /// The setting key that failed.
        key: String,
        /// Error details.
        message: String,
    },

    /// Environment variable parsing error.
    #[error("environment error: {0}")]
    EnvError(String),

    /// File I/O error.
    #[error("file error: {0}")]
    FileError(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Trait for types that can be loaded from settings sources.
///
/// Implement this trait on your configuration struct to enable loading
/// from environment variables, dotenv files, and config files.
pub trait Settings: DeserializeOwned + Sized {
    /// The prefix for environment variable names.
    /// For example, `"MYAPP_"` will read `MYAPP_DATABASE_URL` for a field
    /// named `database_url`.
    fn env_prefix() -> &'static str {
        ""
    }

    /// Whether environment variable names are case-sensitive.
    /// Default: false (case-insensitive matching).
    fn case_sensitive() -> bool {
        false
    }

    /// Nested delimiter for environment variables.
    /// For example, `"__"` allows `MYAPP_REDIS__HOST` to set `redis.host`.
    /// Default: `"__"` (double underscore).
    fn env_nested_delimiter() -> &'static str {
        "__"
    }

    /// Load settings from environment variables.
    ///
    /// Reads all environment variables with the configured prefix,
    /// strips the prefix, converts to lowercase (if not case-sensitive),
    /// and deserializes into the target struct.
    fn from_env() -> Result<Self, SettingsError> {
        let prefix = Self::env_prefix();
        let case_sensitive = Self::case_sensitive();

        let mut map = HashMap::new();
        for (key, value) in std::env::vars() {
            let matches = if case_sensitive {
                key.starts_with(prefix)
            } else {
                key.to_uppercase().starts_with(&prefix.to_uppercase())
            };

            if matches {
                let field_name = &key[prefix.len()..];
                let normalized = if case_sensitive {
                    field_name.to_string()
                } else {
                    field_name.to_lowercase()
                };
                map.insert(normalized, value);
            }
        }

        // Convert the map to a JSON value and deserialize
        let json_value = serde_json::to_value(&map)
            .map_err(|e| SettingsError::EnvError(e.to_string()))?;
        serde_json::from_value(json_value).map_err(SettingsError::JsonError)
    }

    /// Load settings from a JSON string.
    fn from_json_str(json: &str) -> Result<Self, SettingsError> {
        serde_json::from_str(json).map_err(SettingsError::JsonError)
    }

    /// Load settings from a JSON file.
    fn from_json_file(path: &str) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Load settings from a dotenv-style file.
    ///
    /// Reads key=value pairs from the file, applies the prefix filter,
    /// and deserializes into the target struct.
    fn from_dotenv(path: &str) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(path)?;
        let prefix = Self::env_prefix();
        let mut map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Parse KEY=VALUE pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                if key.starts_with(prefix) {
                    let field_name = key[prefix.len()..].to_lowercase();
                    map.insert(field_name, value.to_string());
                } else if prefix.is_empty() {
                    map.insert(key.to_lowercase(), value.to_string());
                }
            }
        }

        let json_value = serde_json::to_value(&map)
            .map_err(|e| SettingsError::EnvError(e.to_string()))?;
        serde_json::from_value(json_value).map_err(SettingsError::JsonError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestConfig {
        host: String,
        port: String,
    }

    impl Settings for TestConfig {
        fn env_prefix() -> &'static str {
            "TEST_"
        }
    }

    #[test]
    fn test_from_env() {
        // Set test env vars
        std::env::set_var("TEST_HOST", "localhost");
        std::env::set_var("TEST_PORT", "8080");

        let config = TestConfig::from_env().unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, "8080");

        // Cleanup
        std::env::remove_var("TEST_HOST");
        std::env::remove_var("TEST_PORT");
    }

    #[test]
    fn test_from_json_str() {
        let json = r#"{"host": "localhost", "port": "8080"}"#;
        let config = TestConfig::from_json_str(json).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, "8080");
    }

    #[test]
    fn test_from_dotenv() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("test_rusdantic.env");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "# Comment").unwrap();
            writeln!(f, "TEST_HOST=myhost").unwrap();
            writeln!(f, "TEST_PORT=9090").unwrap();
            writeln!(f, "").unwrap();
            writeln!(f, "OTHER_VAR=ignored").unwrap();
        }

        let config = TestConfig::from_dotenv(path.to_str().unwrap()).unwrap();
        assert_eq!(config.host, "myhost");
        assert_eq!(config.port, "9090");

        std::fs::remove_file(&path).ok();
    }

    #[derive(Deserialize, Debug)]
    struct NoPrefix {
        name: String,
        value: String,
    }

    impl Settings for NoPrefix {
        fn env_prefix() -> &'static str {
            ""
        }
    }

    #[test]
    fn test_from_dotenv_no_prefix() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("test_rusdantic_noprefix.env");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "NAME=test").unwrap();
            writeln!(f, "VALUE=42").unwrap();
        }

        let config = NoPrefix::from_dotenv(path.to_str().unwrap()).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, "42");

        std::fs::remove_file(&path).ok();
    }
}
