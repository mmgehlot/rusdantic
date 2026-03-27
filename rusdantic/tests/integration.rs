//! Integration tests for the Rusdantic derive macro.
//!
//! These tests verify the full pipeline: derive macro expansion → code generation
//! → runtime validation → error reporting.

use rusdantic::prelude::*;

// =============================================================================
// Basic struct validation
// =============================================================================

#[derive(Rusdantic, Debug)]
struct BasicUser {
    #[rusdantic(length(min = 3, max = 20))]
    username: String,

    #[rusdantic(email)]
    email: String,

    #[rusdantic(range(min = 18))]
    age: u8,
}

#[test]
fn test_basic_valid_struct() {
    let user = BasicUser {
        username: "rust_ace".to_string(),
        email: "user@example.com".to_string(),
        age: 25,
    };
    assert!(user.validate().is_ok());
}

#[test]
fn test_basic_invalid_struct_collects_all_errors() {
    let user = BasicUser {
        username: "ab".to_string(),           // too short (min 3)
        email: "not-an-email".to_string(),    // invalid email
        age: 16,                               // below min (18)
    };
    let err = user.validate().unwrap_err();
    assert_eq!(err.len(), 3);
}

#[test]
fn test_error_paths() {
    let user = BasicUser {
        username: "ab".to_string(),
        email: "valid@example.com".to_string(),
        age: 25,
    };
    let err = user.validate().unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].path_string(), "username");
    assert_eq!(err.errors()[0].code, "length_min");
}

// =============================================================================
// Option<T> handling
// =============================================================================

#[derive(Rusdantic, Debug)]
struct OptionalFields {
    #[rusdantic(length(min = 1))]
    name: String,

    #[rusdantic(email)]
    backup_email: Option<String>,

    #[rusdantic(range(min = 0))]
    score: Option<i32>,
}

#[test]
fn test_option_none_skips_validation() {
    let data = OptionalFields {
        name: "test".to_string(),
        backup_email: None,
        score: None,
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_option_some_validates() {
    let data = OptionalFields {
        name: "test".to_string(),
        backup_email: Some("invalid".to_string()),
        score: Some(-5),
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.len(), 2);
}

#[test]
fn test_option_some_valid() {
    let data = OptionalFields {
        name: "test".to_string(),
        backup_email: Some("valid@example.com".to_string()),
        score: Some(100),
    };
    assert!(data.validate().is_ok());
}

// =============================================================================
// Required on Option<T>
// =============================================================================

#[derive(Rusdantic, Debug)]
struct RequiredOption {
    #[rusdantic(required, email)]
    email: Option<String>,
}

#[test]
fn test_required_option_none_fails() {
    let data = RequiredOption { email: None };
    let err = data.validate().unwrap_err();
    assert!(err.errors().iter().any(|e| e.code == "required"));
}

#[test]
fn test_required_option_some_valid() {
    let data = RequiredOption {
        email: Some("user@example.com".to_string()),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_required_option_some_invalid_email() {
    let data = RequiredOption {
        email: Some("not-email".to_string()),
    };
    let err = data.validate().unwrap_err();
    assert!(err.errors().iter().any(|e| e.code == "email"));
}

// =============================================================================
// Nested struct validation
// =============================================================================

#[derive(Rusdantic, Debug, Clone)]
struct Address {
    #[rusdantic(length(min = 1))]
    street: String,

    #[rusdantic(pattern(regex = r"^\d{5}$"))]
    zip_code: String,
}

#[derive(Rusdantic, Debug)]
struct UserWithAddress {
    #[rusdantic(length(min = 1))]
    name: String,

    #[rusdantic(nested)]
    address: Address,
}

#[test]
fn test_nested_valid() {
    let user = UserWithAddress {
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    assert!(user.validate().is_ok());
}

#[test]
fn test_nested_invalid_reports_full_path() {
    let user = UserWithAddress {
        name: "Alice".to_string(),
        address: Address {
            street: "".to_string(),
            zip_code: "abc".to_string(),
        },
    };
    let err = user.validate().unwrap_err();
    assert_eq!(err.len(), 2);

    // Check that errors have nested paths
    let paths: Vec<String> = err.errors().iter().map(|e| e.path_string()).collect();
    assert!(paths.contains(&"address.street".to_string()));
    assert!(paths.contains(&"address.zip_code".to_string()));
}

// =============================================================================
// Collection validation
// =============================================================================

#[derive(Rusdantic, Debug)]
struct CollectionFields {
    #[rusdantic(length(min = 1, max = 5))]
    tags: Vec<String>,
}

#[test]
fn test_collection_length_valid() {
    let data = CollectionFields {
        tags: vec!["a".to_string(), "b".to_string()],
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_collection_too_few() {
    let data = CollectionFields { tags: vec![] };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].code, "length_min");
}

#[test]
fn test_collection_too_many() {
    let data = CollectionFields {
        tags: vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
        ],
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].code, "length_max");
}

// =============================================================================
// Nested collection validation (Vec<T> where T: Validate)
// =============================================================================

#[derive(Rusdantic, Debug)]
struct UserWithAddresses {
    #[rusdantic(nested)]
    addresses: Vec<Address>,
}

#[test]
fn test_nested_collection_valid() {
    let user = UserWithAddresses {
        addresses: vec![
            Address {
                street: "123 Main".to_string(),
                zip_code: "12345".to_string(),
            },
            Address {
                street: "456 Oak".to_string(),
                zip_code: "67890".to_string(),
            },
        ],
    };
    assert!(user.validate().is_ok());
}

#[test]
fn test_nested_collection_invalid_reports_indexed_paths() {
    let user = UserWithAddresses {
        addresses: vec![
            Address {
                street: "valid".to_string(),
                zip_code: "12345".to_string(),
            },
            Address {
                street: "".to_string(), // invalid
                zip_code: "bad".to_string(), // invalid
            },
        ],
    };
    let err = user.validate().unwrap_err();
    assert_eq!(err.len(), 2);

    let paths: Vec<String> = err.errors().iter().map(|e| e.path_string()).collect();
    assert!(paths.contains(&"addresses[1].street".to_string()));
    assert!(paths.contains(&"addresses[1].zip_code".to_string()));
}

// =============================================================================
// Custom validator
// =============================================================================

fn validate_no_spaces(value: &String) -> Result<(), ValidationError> {
    if value.contains(' ') {
        Err(ValidationError::new("no_spaces", "must not contain spaces"))
    } else {
        Ok(())
    }
}

#[derive(Rusdantic, Debug)]
struct CustomValidated {
    #[rusdantic(custom(function = validate_no_spaces))]
    slug: String,
}

#[test]
fn test_custom_validator_passes() {
    let data = CustomValidated {
        slug: "hello-world".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_custom_validator_fails() {
    let data = CustomValidated {
        slug: "hello world".to_string(),
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].code, "no_spaces");
}

// =============================================================================
// Multiple validators on one field
// =============================================================================

#[derive(Rusdantic, Debug)]
struct MultiValidator {
    #[rusdantic(length(min = 3, max = 50), contains(value = "@"))]
    identifier: String,
}

#[test]
fn test_multiple_validators_all_pass() {
    let data = MultiValidator {
        identifier: "user@domain".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_multiple_validators_both_fail() {
    let data = MultiValidator {
        identifier: "ab".to_string(), // too short AND no @
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.len(), 2);
}

// =============================================================================
// URL validation
// =============================================================================

#[derive(Rusdantic, Debug)]
struct WithUrl {
    #[rusdantic(url)]
    website: String,
}

#[test]
fn test_url_valid() {
    let data = WithUrl {
        website: "https://example.com".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_url_invalid() {
    let data = WithUrl {
        website: "not-a-url".to_string(),
    };
    assert!(data.validate().is_err());
}

// =============================================================================
// Contains validation
// =============================================================================

#[derive(Rusdantic, Debug)]
struct WithContains {
    #[rusdantic(contains(value = "rust"))]
    bio: String,
}

#[test]
fn test_contains_valid() {
    let data = WithContains {
        bio: "I love rust programming".to_string(),
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_contains_invalid() {
    let data = WithContains {
        bio: "I love python".to_string(),
    };
    assert!(data.validate().is_err());
}

// =============================================================================
// Partial validation
// =============================================================================

#[test]
fn test_partial_validation_specific_fields() {
    let user = BasicUser {
        username: "ab".to_string(),           // invalid
        email: "not-email".to_string(),       // invalid
        age: 25,                               // valid
    };

    // Only validate the username field
    let err = user.validate_partial(&["username"]).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err.errors()[0].path_string(), "username");

    // Validate age only — should pass
    assert!(user.validate_partial(&["age"]).is_ok());
}

// =============================================================================
// Error serialization
// =============================================================================

#[test]
fn test_errors_serialize_to_json() {
    let user = BasicUser {
        username: "ab".to_string(),
        email: "bad".to_string(),
        age: 16,
    };
    let err = user.validate().unwrap_err();
    let json = serde_json::to_value(&err).unwrap();

    // Should have 3 errors in the array
    assert_eq!(json["errors"].as_array().unwrap().len(), 3);

    // Each error should have path, code, message
    let first = &json["errors"][0];
    assert!(first.get("path").is_some());
    assert!(first.get("code").is_some());
    assert!(first.get("message").is_some());
}

// =============================================================================
// JSON deserialization with validation
// =============================================================================

#[test]
fn test_from_json_valid() {
    let json = r#"{"username": "rust_ace", "email": "user@example.com", "age": 25}"#;
    let result: Result<BasicUser, _> = rusdantic::from_json(json);
    assert!(result.is_ok());
}

#[test]
fn test_from_json_invalid_data_returns_validation_error() {
    let json = r#"{"username": "ab", "email": "bad", "age": 16}"#;
    let result: Result<BasicUser, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_from_json_malformed_json() {
    let json = r#"{"username": "test", "email":}"#;
    let result: Result<BasicUser, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_from_json_missing_field() {
    let json = r#"{"username": "test", "email": "test@example.com"}"#;
    let result: Result<BasicUser, _> = rusdantic::from_json(json);
    assert!(result.is_err()); // missing "age" field
}

#[test]
fn test_from_value_valid() {
    let value = serde_json::json!({
        "username": "rust_ace",
        "email": "user@example.com",
        "age": 25
    });
    let result: Result<BasicUser, _> = rusdantic::from_value(value);
    assert!(result.is_ok());
}

// =============================================================================
// Serialization roundtrip
// =============================================================================

#[test]
fn test_serialize_roundtrip() {
    let user = BasicUser {
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        age: 25,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&user).unwrap();

    // Deserialize back (with validation)
    let deserialized: BasicUser = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.username, "test_user");
    assert_eq!(deserialized.email, "test@example.com");
    assert_eq!(deserialized.age, 25);
}

// =============================================================================
// Rename_all support
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(rename_all = "camelCase")]
struct CamelCaseStruct {
    #[rusdantic(length(min = 1))]
    first_name: String,

    #[rusdantic(length(min = 1))]
    last_name: String,
}

#[test]
fn test_rename_all_camel_case_deserialization() {
    let json = r#"{"firstName": "Alice", "lastName": "Smith"}"#;
    let result: Result<CamelCaseStruct, _> = rusdantic::from_json(json);
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.first_name, "Alice");
}

#[test]
fn test_rename_all_camel_case_serialization() {
    let data = CamelCaseStruct {
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
    };
    let json = serde_json::to_value(&data).unwrap();
    assert!(json.get("firstName").is_some());
    assert!(json.get("lastName").is_some());
}

#[test]
fn test_rename_all_error_paths_use_serialized_names() {
    let data = CamelCaseStruct {
        first_name: "".to_string(), // invalid
        last_name: "Smith".to_string(),
    };
    let err = data.validate().unwrap_err();
    assert_eq!(err.errors()[0].path_string(), "firstName");
}

// =============================================================================
// deny_unknown_fields
// =============================================================================

#[derive(Rusdantic, Debug)]
#[rusdantic(deny_unknown_fields)]
struct StrictStruct {
    #[rusdantic(length(min = 1))]
    name: String,
}

#[test]
fn test_deny_unknown_fields_rejects_extra() {
    let json = r#"{"name": "test", "extra": "field"}"#;
    let result: Result<StrictStruct, _> = rusdantic::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_deny_unknown_fields_accepts_valid() {
    let json = r#"{"name": "test"}"#;
    let result: Result<StrictStruct, _> = rusdantic::from_json(json);
    assert!(result.is_ok());
}

// =============================================================================
// No validators on a field — should still compile and work
// =============================================================================

#[derive(Rusdantic, Debug)]
struct NoValidation {
    name: String,
    value: i32,
}

#[test]
fn test_no_validators_always_valid() {
    let data = NoValidation {
        name: "".to_string(),
        value: -999,
    };
    assert!(data.validate().is_ok());
}

#[test]
fn test_no_validators_serialization_works() {
    let data = NoValidation {
        name: "test".to_string(),
        value: 42,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"name\":\"test\""));
}

// =============================================================================
// JSON Schema generation
// =============================================================================

#[test]
fn test_json_schema_basic() {
    let schema = BasicUser::json_schema();
    assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(schema["title"], "BasicUser");
    assert_eq!(schema["type"], "object");

    // Check properties exist
    let props = schema["properties"].as_object().unwrap();
    assert!(props.contains_key("username"));
    assert!(props.contains_key("email"));
    assert!(props.contains_key("age"));

    // Check required fields
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("username")));
    assert!(required.contains(&serde_json::json!("email")));
    assert!(required.contains(&serde_json::json!("age")));
}

#[test]
fn test_json_schema_string_constraints() {
    let schema = BasicUser::json_schema();
    let username_schema = &schema["properties"]["username"];
    assert_eq!(username_schema["type"], "string");
    assert_eq!(username_schema["minLength"], 3);
    assert_eq!(username_schema["maxLength"], 20);
}

#[test]
fn test_json_schema_email_format() {
    let schema = BasicUser::json_schema();
    let email_schema = &schema["properties"]["email"];
    assert_eq!(email_schema["format"], "email");
}

#[test]
fn test_json_schema_optional_field_nullable() {
    let schema = OptionalFields::json_schema();
    let backup_email = &schema["properties"]["backup_email"];
    // Should be anyOf: [{...}, {type: "null"}]
    assert!(backup_email.get("anyOf").is_some());
}

// =============================================================================
// Generic struct support
// =============================================================================
// TODO: Generic struct tests temporarily moved to a separate file to prevent
// compilation errors from blocking all other integration tests.
// See tests/generics.rs for generic struct support tests.
