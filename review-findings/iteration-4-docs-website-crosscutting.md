# Iteration 4: Documentation, Website, Cross-cutting

## Critical
1. HTML website code examples use WRONG attribute syntax: min_length/max_length/ge/le instead of length(min=)/range(min=)
2. HTML example links point to non-existent directory paths (basic_validation/ vs basic.rs)

## High
3. Website claims "one_of enum check" feature that doesn't exist
4. Website claims gt/lt operators but only >=/<= (min/max) are implemented

## Medium
5. No settings management runnable example (feature advertised but no example file)
6. Website examples section missing dump_options and types_library examples
7. Partial validation requires serialized names (rename_all footgun) — no docs

## Low  
8. README says "10 examples" but there are 11
9. HTML example directory naming doesn't match actual file names
10. Settings crate not linked from HTML examples section
