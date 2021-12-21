use std::sync::Arc;

use crate::{
    flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
    init_thread_stream,
    panic_hook::{init_ctrlc_hook, init_panic_hook},
    shutdown_event_dispatch, EventSink,
};

pub struct TelemetrySystemGuard {}

impl TelemetrySystemGuard {
    pub fn new(sink: Arc<dyn EventSink>) -> anyhow::Result<Self, String> {
        init_telemetry(sink)?;
        Ok(Self {})
    }
}

impl std::ops::Drop for TelemetrySystemGuard {
    fn drop(&mut self) {
        shutdown_telemetry();
    }
}

pub fn init_telemetry(sink: Arc<dyn EventSink>) -> anyhow::Result<(), String> {
    init_event_dispatch(10 * 1024 * 1024, 10 * 1024 * 1024, 1024 * 1024, sink)?;
    init_panic_hook();
    init_ctrlc_hook();
    Ok(())
}

pub fn shutdown_telemetry() {
    flush_log_buffer();
    flush_metrics_buffer();
    shutdown_event_dispatch();
}

pub struct TelemetryThreadGuard {
    _dummy_ptr: *mut u8,
}

impl TelemetryThreadGuard {
    pub fn new() -> Self {
        init_thread_stream();
        Self {
            _dummy_ptr: std::ptr::null_mut(),
        }
    }
}

impl std::ops::Drop for TelemetryThreadGuard {
    fn drop(&mut self) {
        flush_thread_buffer();
    }
}

//not used at the time of writing, but clippy wants it
impl Default for TelemetryThreadGuard {
    fn default() -> Self {
        Self::new()
    }
}
