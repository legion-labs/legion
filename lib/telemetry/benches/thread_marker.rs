use criterion::{criterion_group, criterion_main, Criterion};
use lgn_telemetry::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty_scope", |b| {
        b.iter(|| {
            trace_scope!();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
