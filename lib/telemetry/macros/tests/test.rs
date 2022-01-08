use std::sync::Arc;

use lgn_telemetry::{NullEventSink, TelemetrySystemGuard, TelemetryThreadGuard};
use lgn_telemetry_proc_macros::{log_function, trace_function};

#[trace_function]
fn trace_func() {}

#[trace_function("foo")]
fn trace_func_named() {}

#[log_function]
fn log_func() {}

#[test]
fn test_macros() {
    let _telemetry_guard = TelemetrySystemGuard::new(Arc::new(NullEventSink {}));
    let _thread_guard = TelemetryThreadGuard::new();

    trace_func();

    trace_func_named();

    log_func();
}
