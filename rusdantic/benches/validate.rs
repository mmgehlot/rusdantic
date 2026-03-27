//! Benchmarks for Rusdantic validation performance.
//! Run with: `cargo bench --bench validate`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusdantic::prelude::*;

#[derive(Rusdantic, Debug, Clone)]
struct SimpleUser {
    #[rusdantic(length(min = 3, max = 20))]
    username: String,
    #[rusdantic(email)]
    email: String,
    #[rusdantic(range(min = 18))]
    age: u8,
}

#[derive(Rusdantic, Debug, Clone)]
struct Address {
    #[rusdantic(length(min = 1))]
    street: String,
    #[rusdantic(pattern(regex = r"^\d{5}$"))]
    zip_code: String,
}

#[derive(Rusdantic, Debug, Clone)]
struct NestedUser {
    #[rusdantic(length(min = 1))]
    name: String,
    #[rusdantic(nested)]
    address: Address,
}

fn bench_validate_simple(c: &mut Criterion) {
    let valid = SimpleUser {
        username: "rust_ace".to_string(),
        email: "user@example.com".to_string(),
        age: 25,
    };
    let invalid = SimpleUser {
        username: "ab".to_string(),
        email: "bad".to_string(),
        age: 16,
    };

    c.bench_function("validate_simple_valid", |b| {
        b.iter(|| black_box(&valid).validate())
    });
    c.bench_function("validate_simple_invalid", |b| {
        b.iter(|| black_box(&invalid).validate())
    });
}

fn bench_validate_nested(c: &mut Criterion) {
    let valid = NestedUser {
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            zip_code: "12345".to_string(),
        },
    };

    c.bench_function("validate_nested_valid", |b| {
        b.iter(|| black_box(&valid).validate())
    });
}

fn bench_from_json(c: &mut Criterion) {
    let json_valid = r#"{"username":"rust_ace","email":"user@example.com","age":25}"#;
    let json_invalid = r#"{"username":"ab","email":"bad","age":16}"#;

    c.bench_function("from_json_valid", |b| {
        b.iter(|| rusdantic::from_json::<SimpleUser>(black_box(json_valid)))
    });
    c.bench_function("from_json_invalid", |b| {
        b.iter(|| rusdantic::from_json::<SimpleUser>(black_box(json_invalid)))
    });
}

fn bench_serialize(c: &mut Criterion) {
    let user = SimpleUser {
        username: "rust_ace".to_string(),
        email: "user@example.com".to_string(),
        age: 25,
    };

    c.bench_function("serialize_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&user)))
    });
}

criterion_group!(
    benches,
    bench_validate_simple,
    bench_validate_nested,
    bench_from_json,
    bench_serialize,
);
criterion_main!(benches);
