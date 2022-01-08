use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use lgn_tracing::{prelude::*, NullEventSink};

pub fn criterion_benchmark(c: &mut Criterion) {
    let _telemetry_guard = TelemetrySystemGuard::new(Arc::new(NullEventSink {}));
    let _thread_guard = TelemetryThreadGuard::new();

    c.bench_function("dispatch/log", |b| {
        b.iter(|| {
            error!("test");
        })
    });
    static METRIC: MetricDesc = MetricDesc {
        name: "name",
        unit: "unit",
    };
    c.bench_function("dispatch/metric", |b| {
        b.iter(|| {
            record_int_metric(&METRIC, 0);
        })
    });
    c.bench_function("dispatch/trace_scope", |b| {
        b.iter(|| {
            trace_scope!("test");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
