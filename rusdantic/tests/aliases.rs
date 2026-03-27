//! Tests for field alias support.
//!
//! Aliases allow fields to have different names for deserialization (input)
//! and serialization (output). When an alias is set, BOTH the alias and
//! the original Rust field name (after rename_all) are accepted during
//! deserialization.

use rusdantic::prelude::*;

// =============================================================================
// Basic alias
// =============================================================================

#[derive(Rusdantic, Debug)]
struct WithAlias {
    #[rusdantic(alias = "userName", length(min = 1))]
    user_name: String,

    #[rusdantic(email)]
    email: String,
}

#[test]
fn test_alias_accepted_in_deserialization() {
    let json = r#"{"userName": "alice", "email": "a@b.com"}"#;
    let result: WithAlias = rusdantic::from_json(json).unwrap();
    assert_eq!(result.user_name, "alice");
}

#[test]
fn test_original_name_also_accepted() {
    // When alias is set, the original Rust field name is also accepted
    let json = r#"{"user_name": "bob", "email": "b@c.com"}"#;
    let result: WithAlias = rusdantic::from_json(json).unwrap();
    assert_eq!(result.user_name, "bob");
}

#[test]
fn test_alias_used_in_serialization() {
    let data = WithAlias {
        user_name: "alice".to_string(),
        email: "a@b.com".to_string(),
    };
    let json = serde_json::to_value(&data).unwrap();
    // Serialization uses the alias as the key
    assert!(json.get("userName").is_some());
    assert!(json.get("user_name").is_none());
}

#[test]
fn test_alias_error_path_uses_alias() {
    let data = WithAlias {
        user_name: "".to_string(), // invalid: length(min=1)
        email: "a@b.com".to_string(),
    };
    let err = data.validate().unwrap_err();
    // Error path should use the alias name (what the API consumer sees)
    assert_eq!(err.errors()[0].path_string(), "userName");
}

// =============================================================================
// Separate validation_alias and serialization_alias
// =============================================================================

#[derive(Rusdantic, Debug)]
struct SeparateAliases {
    #[rusdantic(
        validation_alias = "user_input_name",
        serialization_alias = "displayName",
        length(min = 1)
    )]
    name: String,
}

#[test]
fn test_validation_alias_accepted() {
    let json = r#"{"user_input_name": "alice"}"#;
    let result: SeparateAliases = rusdantic::from_json(json).unwrap();
    assert_eq!(result.name, "alice");
}

#[test]
fn test_serialization_alias_used_in_output() {
    let data = SeparateAliases {
        name: "alice".to_string(),
    };
    let json = serde_json::to_value(&data).unwrap();
    assert!(json.get("displayName").is_some());
}

// =============================================================================
// Alias with rename_all
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(rename_all = "camelCase")]
struct AliasWithRenameAll {
    // alias overrides rename_all for this field
    #[rusdantic(alias = "USERNAME")]
    user_name: String,

    // No alias: uses rename_all → "lastName"
    last_name: String,
}

#[test]
fn test_alias_overrides_rename_all() {
    let json = r#"{"USERNAME": "alice", "lastName": "Smith"}"#;
    let result: AliasWithRenameAll = rusdantic::from_json(json).unwrap();
    assert_eq!(result.user_name, "alice");
    assert_eq!(result.last_name, "Smith");
}

#[test]
fn test_rename_all_name_also_accepted_as_alias() {
    // When alias is set, the rename_all version is also accepted
    let json = r#"{"userName": "bob", "lastName": "Jones"}"#;
    let result: AliasWithRenameAll = rusdantic::from_json(json).unwrap();
    assert_eq!(result.user_name, "bob");
}

#[test]
fn test_alias_field_serializes_with_alias() {
    let data = AliasWithRenameAll {
        user_name: "alice".to_string(),
        last_name: "Smith".to_string(),
    };
    let json = serde_json::to_value(&data).unwrap();
    assert!(json.get("USERNAME").is_some()); // alias field uses alias
    assert!(json.get("lastName").is_some()); // non-alias field uses rename_all
}
