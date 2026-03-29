# Iteration 3: Codegen, Examples, CI/CD

## High
1. JSON Schema: Range validators produce NO minimum/maximum constraints (runtime expressions can't be evaluated at compile time)
2. Enum support: Only Validate impl generated — no Serialize/Deserialize, breaking validate-on-deserialize promise

## Medium
3. CI: No --no-default-features testing (url-validation could silently be disabled)
4. Generic structs: json_schema() and validate_partial() silently not generated — confusing "method not found" error
5. Schema: Custom/Contains/Required validators have no schema representation (no x-extension either)
6. Empty UI test directories — zero compile-time error test coverage for diagnostics.rs
7. Partial validation requires serialized names (camelCase if rename_all), undocumented footgun

## Low
8. CI: No cargo test --doc execution
9. Sanitizer codegen asymmetry between Option and non-Option (maintainability)
10. Dependency version pinning too loose (serde = "1" instead of "1.0")
11. Range schema handler ignores max explicitly (asymmetric with Length which IS rendered)
12. Enum tuple variants silently skip validation
