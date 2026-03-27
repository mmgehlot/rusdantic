//! Benchmarks for Rusdantic validation performance.
//! Run with: `cargo bench --bench validate`

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_placeholder(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // Benchmarks will be populated once the derive macro is functional
            std::hint::black_box(42)
        })
    });
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
