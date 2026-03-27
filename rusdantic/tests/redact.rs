//! Tests for PII redaction in Debug output.

use rusdantic::prelude::*;

#[derive(Rusdantic)]
struct UserWithPII {
    name: String,

    #[rusdantic(redact)]
    email: String,

    #[rusdantic(redact(with = "***"))]
    ssn: String,

    #[rusdantic(redact(hash))]
    api_key: String,
}

#[test]
fn test_redact_default_shows_redacted() {
    let user = UserWithPII {
        name: "Alice".to_string(),
        email: "alice@secret.com".to_string(),
        ssn: "123-45-6789".to_string(),
        api_key: "sk-secret-key-123".to_string(),
    };
    let debug = format!("{:?}", user);

    // Name should be visible (not redacted)
    assert!(debug.contains("Alice"));

    // Email should show [REDACTED]
    assert!(!debug.contains("alice@secret.com"));
    assert!(debug.contains("[REDACTED]"));

    // SSN should show custom replacement
    assert!(!debug.contains("123-45-6789"));
    assert!(debug.contains("***"));

    // API key should show a hash
    assert!(!debug.contains("sk-secret-key-123"));
    assert!(debug.contains("[HASH:"));
}

#[test]
fn test_redact_hash_is_deterministic() {
    let user1 = UserWithPII {
        name: "Alice".to_string(),
        email: "a@b.com".to_string(),
        ssn: "000".to_string(),
        api_key: "same-key".to_string(),
    };
    let user2 = UserWithPII {
        name: "Bob".to_string(),
        email: "c@d.com".to_string(),
        ssn: "111".to_string(),
        api_key: "same-key".to_string(),
    };
    let debug1 = format!("{:?}", user1);
    let debug2 = format!("{:?}", user2);

    // Extract hash values - they should be the same since api_key is the same
    let hash1: String = debug1
        .split("[HASH:")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .to_string();
    let hash2: String = debug2
        .split("[HASH:")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .to_string();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_redact_different_values_different_hashes() {
    let user1 = UserWithPII {
        name: "A".to_string(),
        email: "a@b.com".to_string(),
        ssn: "000".to_string(),
        api_key: "key-one".to_string(),
    };
    let user2 = UserWithPII {
        name: "B".to_string(),
        email: "c@d.com".to_string(),
        ssn: "111".to_string(),
        api_key: "key-two".to_string(),
    };
    let debug1 = format!("{:?}", user1);
    let debug2 = format!("{:?}", user2);

    let hash1: String = debug1
        .split("[HASH:")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .to_string();
    let hash2: String = debug2
        .split("[HASH:")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .to_string();
    assert_ne!(hash1, hash2);
}
