# Iteration 1: Core Library + Derive Macro

## Critical
1. PII redaction hash uses DefaultHasher (not cryptographic) — need SHA2 or document as correlation-only
2. Generic type Deserialize silently skipped — should generate compile error

## Medium  
3. Float coercion missing NaN/Infinity check (is_finite)
4. ValidatorMode::Before parsed but never implemented — should error at compile time
5. Custom validator panics propagate unhandled — need catch_unwind or documentation
6. Custom sanitizer function signature not validated — confusing compile errors
7. Duplicate field handling silent when aliases used + deny_unknown_fields=false

## Low
8. Partial validation lacks strict/lenient mode toggle
9. Coercion error messages are string-only (not structured)
10. Regex codegen pattern duplicated in deserialize.rs and validate.rs
11. Sanitizer Option<T> handling duplicated
12. Collection nested validation assumes 0-based indexing
13. Computed fields cannot be validated
14. Inconsistent error code naming (email vs length_min)
15. "Rusdantic" capitalization inconsistent in messages
16. dump.rs String::from_utf8 unwrap needs version comment
17. anchor_pattern handles ^$ redundantly (cosmetic)
