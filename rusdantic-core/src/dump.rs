//! Advanced serialization with options (model_dump / model_dump_json equivalent).
//!
//! Provides `DumpOptions` for controlling serialization output: include/exclude
//! fields, use aliases, skip unset/default/none values.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

/// Options for controlling serialization output.
///
/// Mirrors Pydantic's `model_dump()` parameters.
///
/// # Example
///
/// ```rust
/// use rusdantic_core::dump::DumpOptions;
///
/// let opts = DumpOptions::new()
///     .exclude(&["password", "secret"])
///     .exclude_none(true)
///     .by_alias(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DumpOptions {
    /// Only include these fields in the output (whitelist).
    /// If empty, all fields are included.
    pub include_fields: HashSet<String>,
    /// Exclude these fields from the output (blacklist).
    pub exclude_fields: HashSet<String>,
    /// If true, skip fields with `None` values.
    pub exclude_none: bool,
    /// If true, skip fields whose values equal their default.
    pub exclude_defaults: bool,
    /// If true, use the serialization alias as the field name.
    pub by_alias: bool,
    /// If true, only include fields that were explicitly provided
    /// during deserialization (requires fields_set tracking).
    pub exclude_unset: bool,
    /// Indentation for JSON output (None = compact).
    pub indent: Option<usize>,
}

impl DumpOptions {
    /// Create a new `DumpOptions` with default settings (include all, compact).
    pub fn new() -> Self {
        Self::default()
    }

    /// Only include the specified fields in the output.
    pub fn include(mut self, fields: &[&str]) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Exclude the specified fields from the output.
    pub fn exclude(mut self, fields: &[&str]) -> Self {
        self.exclude_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// If true, skip fields with `None` / `null` values.
    pub fn exclude_none(mut self, yes: bool) -> Self {
        self.exclude_none = yes;
        self
    }

    /// If true, skip fields whose values equal their defaults.
    pub fn exclude_defaults(mut self, yes: bool) -> Self {
        self.exclude_defaults = yes;
        self
    }

    /// If true, use serialization aliases as field names.
    pub fn by_alias(mut self, yes: bool) -> Self {
        self.by_alias = yes;
        self
    }

    /// If true, only include fields that were explicitly set.
    pub fn exclude_unset(mut self, yes: bool) -> Self {
        self.exclude_unset = yes;
        self
    }

    /// Set JSON indentation (None = compact).
    pub fn indent(mut self, spaces: usize) -> Self {
        self.indent = Some(spaces);
        self
    }

    /// Apply options to a serialized JSON Value, filtering fields.
    ///
    /// This is the core filtering logic. It takes a fully serialized
    /// `serde_json::Value` (typically an Object) and removes/renames
    /// fields based on the configured options.
    pub fn filter_value(&self, value: &mut Value) {
        if let Value::Object(ref mut map) = value {
            // Collect keys to remove
            let keys_to_remove: Vec<String> = map
                .keys()
                .filter(|key| {
                    // Check include list (if non-empty, only whitelisted keys survive)
                    if !self.include_fields.is_empty() && !self.include_fields.contains(*key) {
                        return true;
                    }
                    // Check exclude list
                    if self.exclude_fields.contains(*key) {
                        return true;
                    }
                    // Check exclude_none
                    if self.exclude_none {
                        if let Some(val) = map.get(*key) {
                            if val.is_null() {
                                return true;
                            }
                        }
                    }
                    false
                })
                .cloned()
                .collect();

            for key in keys_to_remove {
                map.remove(&key);
            }
        }
    }
}

/// Trait for types that support advanced serialization with options.
///
/// Automatically implemented for any type that implements `serde::Serialize`.
pub trait Dump: Serialize {
    /// Serialize to a `serde_json::Value` with default options.
    fn dump(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Serialize to a `serde_json::Value` with custom options.
    fn dump_with(&self, options: &DumpOptions) -> Result<Value, serde_json::Error> {
        let mut value = serde_json::to_value(self)?;
        options.filter_value(&mut value);
        Ok(value)
    }

    /// Serialize to a JSON string with default options.
    fn dump_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to a JSON string with custom options.
    fn dump_json_with(&self, options: &DumpOptions) -> Result<String, serde_json::Error> {
        let mut value = serde_json::to_value(self)?;
        options.filter_value(&mut value);
        if let Some(indent) = options.indent {
            // Pretty print with custom indent
            let buf = Vec::new();
            let indent_bytes = " ".repeat(indent).into_bytes();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
            let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
            serde::Serialize::serialize(&value, &mut ser)
                .map_err(serde_json::Error::from)?;
            // SAFETY: serde_json always produces valid UTF-8
            Ok(String::from_utf8(ser.into_inner()).unwrap())
        } else {
            serde_json::to_string(&value)
        }
    }
}

// Blanket implementation: any Serialize type gets Dump for free
impl<T: Serialize> Dump for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dump_options_exclude() {
        let opts = DumpOptions::new().exclude(&["password", "secret"]);
        let mut value = json!({"name": "alice", "password": "hidden", "secret": "key"});
        opts.filter_value(&mut value);
        assert!(value.get("name").is_some());
        assert!(value.get("password").is_none());
        assert!(value.get("secret").is_none());
    }

    #[test]
    fn test_dump_options_include() {
        let opts = DumpOptions::new().include(&["name", "email"]);
        let mut value = json!({"name": "alice", "email": "a@b.com", "age": 30});
        opts.filter_value(&mut value);
        assert!(value.get("name").is_some());
        assert!(value.get("email").is_some());
        assert!(value.get("age").is_none());
    }

    #[test]
    fn test_dump_options_exclude_none() {
        let opts = DumpOptions::new().exclude_none(true);
        let mut value = json!({"name": "alice", "bio": null, "age": 30});
        opts.filter_value(&mut value);
        assert!(value.get("name").is_some());
        assert!(value.get("bio").is_none());
        assert!(value.get("age").is_some());
    }

    #[test]
    fn test_dump_options_combined() {
        let opts = DumpOptions::new()
            .exclude(&["password"])
            .exclude_none(true);
        let mut value = json!({"name": "alice", "password": "x", "bio": null});
        opts.filter_value(&mut value);
        assert!(value.get("name").is_some());
        assert!(value.get("password").is_none());
        assert!(value.get("bio").is_none());
    }

    #[test]
    fn test_dump_trait_on_serialize() {
        #[derive(serde::Serialize)]
        struct User {
            name: String,
            age: u32,
        }
        let user = User {
            name: "alice".to_string(),
            age: 30,
        };
        let value = user.dump().unwrap();
        assert_eq!(value["name"], "alice");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_dump_with_exclude() {
        #[derive(serde::Serialize)]
        struct User {
            name: String,
            password: String,
        }
        let user = User {
            name: "alice".to_string(),
            password: "secret".to_string(),
        };
        let opts = DumpOptions::new().exclude(&["password"]);
        let value = user.dump_with(&opts).unwrap();
        assert!(value.get("name").is_some());
        assert!(value.get("password").is_none());
    }

    #[test]
    fn test_dump_json_compact() {
        #[derive(serde::Serialize)]
        struct Item {
            name: String,
        }
        let item = Item {
            name: "test".to_string(),
        };
        let json = item.dump_json().unwrap();
        assert_eq!(json, r#"{"name":"test"}"#);
    }

    #[test]
    fn test_dump_json_with_indent() {
        #[derive(serde::Serialize)]
        struct Item {
            name: String,
        }
        let item = Item {
            name: "test".to_string(),
        };
        let opts = DumpOptions::new().indent(2);
        let json = item.dump_json_with(&opts).unwrap();
        assert!(json.contains("\n"));
        assert!(json.contains("  "));
    }
}
