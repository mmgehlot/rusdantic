//! Built-in validation rules for Rusdantic.
//!
//! Each rule module provides a validation function that the derive macro's
//! generated code calls at runtime. All functions follow a consistent pattern:
//! they take the value to validate, constraint parameters, the field path,
//! and a mutable reference to a `ValidationErrors` collection.
//!
//! This design allows error accumulation (collect-all) rather than
//! fail-fast behavior.

pub mod contains;
pub mod email;
pub mod length;
pub mod pattern;
pub mod range;
pub mod required;
pub mod url;

// Re-export all validation functions at the rules module level
// so generated code can use `::rusdantic_core::rules::validate_length(...)`.
pub use contains::validate_contains;
pub use email::validate_email;
pub use length::validate_length;
pub use pattern::validate_pattern;
pub use range::validate_range;
pub use required::validate_required;
pub use url::validate_url;

/// Trait for types that have a measurable length.
/// Used by the length validator to work with both strings and collections.
pub trait HasLength {
    /// Return the length of this value (character count for strings,
    /// element count for collections).
    fn rusdantic_length(&self) -> usize;
}

impl HasLength for String {
    fn rusdantic_length(&self) -> usize {
        // Use char count, not byte length, for Unicode correctness
        self.chars().count()
    }
}

impl HasLength for &str {
    fn rusdantic_length(&self) -> usize {
        self.chars().count()
    }
}

impl<T> HasLength for Vec<T> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<T: std::hash::Hash + Eq> HasLength for std::collections::HashSet<T> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<T: Ord> HasLength for std::collections::BTreeSet<T> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<K: std::hash::Hash + Eq, V> HasLength for std::collections::HashMap<K, V> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<K: Ord, V> HasLength for std::collections::BTreeMap<K, V> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for std::collections::VecDeque<T> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for std::collections::LinkedList<T> {
    fn rusdantic_length(&self) -> usize {
        self.len()
    }
}

/// Trait for types that can be checked against a string pattern.
/// Used by email, url, pattern, and contains validators.
pub trait AsStr {
    /// Return this value as a string slice for pattern matching.
    fn as_str_ref(&self) -> &str;
}

impl AsStr for String {
    fn as_str_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsStr for &str {
    fn as_str_ref(&self) -> &str {
        self
    }
}

impl AsStr for std::borrow::Cow<'_, str> {
    fn as_str_ref(&self) -> &str {
        self.as_ref()
    }
}
