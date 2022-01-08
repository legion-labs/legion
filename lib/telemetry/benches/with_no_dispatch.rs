use criterion::{criterion_group, criterion_main, Criterion};
use lgn_telemetry::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("no_dispatch/log", |b| {
        b.iter(|| {
            error!("test");
        })
    });
    static METRIC: MetricDesc = MetricDesc {
        name: "name",
        unit: "unit",
    };
    c.bench_function("no_dispatch/metric", |b| {
        b.iter(|| {
            record_int_metric(&METRIC, 0);
        })
    });
    c.bench_function("no_dispatch/trace_scope", |b| {
        b.iter(|| {
            trace_scope!();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
