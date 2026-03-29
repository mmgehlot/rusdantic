# Iteration 5: Security, Edge Cases, Production Hardening

## Medium
1. Range validator: f64::INFINITY passes with single-bound range(min=0) — need is_finite() check
2. No ReDoS analysis on user-supplied regex patterns at compile time
3. Computed fields bypass validation and redaction during serialization — can leak secrets
4. Error messages include unbounded user patterns/needles — potential memory DoS

## Low
5. Email local part length not checked (RFC 5321: max 64 chars)
6. contains(value = "") always passes — should warn at compile time
7. URL fallback validator (without url-validation feature) accepts javascript://, file:///
8. Pattern anchor_pattern non-capturing group not documented (why)
9. PathSegment field names not escaped in path strings (low risk since Rust idents can't contain dots)
