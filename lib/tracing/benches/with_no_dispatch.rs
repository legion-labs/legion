use criterion::{criterion_group, criterion_main, Criterion};
use lgn_tracing::{error, metric_int, trace_scope};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("no_dispatch/log", |b| {
        b.iter(|| {
            error!("test");
        })
    });
    c.bench_function("no_dispatch/metric", |b| {
        b.iter(|| {
            metric_int!("unit", "name", 0);
        })
    });
    c.bench_function("no_dispatch/trace_scope", |b| {
        b.iter(|| {
            trace_scope!("test");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
