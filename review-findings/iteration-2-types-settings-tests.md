# Iteration 2: Types Library + Settings + Tests

## High
1. SecretStr/SecretBytes Hash is non-constant-time — timing side-channel risk when used in HashMap
2. Settings env var parsing has no size limit — DoS via huge env vars
3. Settings from_dotenv has no path validation — path traversal risk
4. Secret<T> derives Clone without zeroize — secret may remain in memory after drop

## Medium
5. HttpUrl accepts any port (0, >65535) and localhost/private IPs (SSRF risk)
6. Settings prefix matching with empty prefix has redundant logic
7. EmailStr redundant @ check before regex
8. Sanitizers only run during from_json(), not .validate() — documented but dangerous
9. Missing test: email boundary 254/255 chars
10. Missing test: numeric type overflow during deserialization
11. Missing test: HttpUrl port validation
12. Missing test: settings coercion edge cases (scientific notation, empty string)
13. Missing test: redaction in Display output, serialization

## Low
14. Settings case-insensitive prefix matching does double string allocation
15. Numeric error messages allocate String per failure (use Cow<str>)
16. FiniteFloat Display missing doc comment explaining safety
17. NonEmptyString accepts whitespace-only strings
