use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use lgn_tracing::{
    error,
    event_sink::NullEventSink,
    guard::{TelemetrySystemGuard, TelemetryThreadGuard},
    metric_int, trace_scope,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    let _telemetry_guard = TelemetrySystemGuard::new(Arc::new(NullEventSink {}));
    let _thread_guard = TelemetryThreadGuard::new();

    c.bench_function("dispatch/log", |b| {
        b.iter(|| {
            error!("test");
        })
    });
    c.bench_function("dispatch/metric", |b| {
        b.iter(|| {
            metric_int!("name", "unit", 0);
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
