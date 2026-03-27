//! TypeAdapter: standalone validation and serialization without BaseModel.
//!
//! Validates and serializes arbitrary types without requiring them to be
//! Rusdantic structs. Equivalent to Pydantic's TypeAdapter.
//!
//! # Example
//!
//! ```rust
//! use rusdantic_core::adapter::TypeAdapter;
//!
//! // Validate a plain vector
//! let adapter = TypeAdapter::<Vec<u32>>::new();
//! let validated: Vec<u32> = adapter.validate_json(r#"[1, 2, 3]"#).unwrap();
//! assert_eq!(validated, vec![1, 2, 3]);
//! ```

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

/// A standalone validator/serializer for any serde-compatible type.
///
/// TypeAdapter provides validation and serialization capabilities
/// without requiring the derive macro. It works with any type that
/// implements the appropriate serde traits.
pub struct TypeAdapter<T> {
    _phantom: PhantomData<T>,
}

impl<T> TypeAdapter<T> {
    /// Create a new TypeAdapter for the given type.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for TypeAdapter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: DeserializeOwned> TypeAdapter<T> {
    /// Validate and deserialize a JSON string.
    pub fn validate_json(&self, json: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate and deserialize a serde_json::Value.
    pub fn validate_value(&self, value: serde_json::Value) -> Result<T, serde_json::Error> {
        serde_json::from_value(value)
    }
}

impl<T: Serialize> TypeAdapter<T> {
    /// Serialize the value to a JSON string.
    pub fn dump_json(&self, value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    /// Serialize the value to a serde_json::Value.
    pub fn dump_value(&self, value: &T) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(value)
    }
}

impl<T: Serialize + DeserializeOwned> TypeAdapter<T> {
    /// Generate a basic JSON Schema for the type.
    ///
    /// This produces a minimal schema based on the JSON representation.
    /// For full schema generation with validation constraints, use the
    /// `json_schema()` method on a Rusdantic-derived struct.
    pub fn json_schema_basic(&self) -> serde_json::Value {
        // Return a permissive schema since we can't introspect
        // serde trait impls at runtime.
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_json_vec() {
        let adapter = TypeAdapter::<Vec<u32>>::new();
        let result = adapter.validate_json("[1, 2, 3]").unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_validate_json_string() {
        let adapter = TypeAdapter::<String>::new();
        let result = adapter.validate_json("\"hello\"").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_validate_json_invalid() {
        let adapter = TypeAdapter::<Vec<u32>>::new();
        assert!(adapter.validate_json("[1, \"not a number\"]").is_err());
    }

    #[test]
    fn test_validate_value() {
        let adapter = TypeAdapter::<Vec<String>>::new();
        let result = adapter.validate_value(json!(["a", "b", "c"])).unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_dump_json() {
        let adapter = TypeAdapter::<Vec<u32>>::new();
        let json = adapter.dump_json(&vec![1, 2, 3]).unwrap();
        assert_eq!(json, "[1,2,3]");
    }

    #[test]
    fn test_dump_value() {
        let adapter = TypeAdapter::<String>::new();
        let value = adapter.dump_value(&"hello".to_string()).unwrap();
        assert_eq!(value, json!("hello"));
    }

    #[test]
    fn test_default() {
        let adapter = TypeAdapter::<i32>::default();
        assert_eq!(adapter.validate_json("42").unwrap(), 42);
    }

    #[test]
    fn test_hashmap() {
        use std::collections::HashMap;
        let adapter = TypeAdapter::<HashMap<String, i32>>::new();
        let result = adapter.validate_json(r#"{"a": 1, "b": 2}"#).unwrap();
        assert_eq!(result.get("a"), Some(&1));
    }
}
