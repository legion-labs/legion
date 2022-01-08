use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use lgn_telemetry::{prelude::*, NullEventSink};

pub fn disabled_scope(c: &mut Criterion) {
    c.bench_function("disabled_scope", |b| {
        b.iter(|| {
            trace_scope!();
        })
    });
}

// This initializes global state form now on the telemetry is going tbe enabled
pub fn enabled_scope(c: &mut Criterion) {
    let _telemetry_guard = TelemetrySystemGuard::new(Arc::new(NullEventSink {}));
    let _thread_guard = TelemetryThreadGuard::new();

    c.bench_function("enabled_scope", |b| {
        b.iter(|| {
            trace_scope!();
        })
    });
}

criterion_group!(benches, disabled_scope, enabled_scope);
criterion_main!(benches);
