use std::{marker::PhantomData, sync::Arc};

use crate::{
    dispatch::{
        flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
        init_thread_stream, on_end_scope, shutdown_dispatch,
    },
    event_sink::EventSink,
    panic_hook::init_panic_hook,
    thread_events::ThreadSpanDesc,
};

pub struct TelemetrySystemGuard {}

impl TelemetrySystemGuard {
    pub fn new(sink: Arc<dyn EventSink>) -> anyhow::Result<Self> {
        init_telemetry(sink)?;
        Ok(Self {})
    }
}

impl std::ops::Drop for TelemetrySystemGuard {
    fn drop(&mut self) {
        shutdown_telemetry();
    }
}

pub fn init_telemetry(sink: Arc<dyn EventSink>) -> anyhow::Result<()> {
    init_event_dispatch(10 * 1024 * 1024, 10 * 1024 * 1024, 1024 * 1024, sink)?;
    init_panic_hook();
    Ok(())
}

pub fn shutdown_telemetry() {
    flush_log_buffer();
    flush_metrics_buffer();
    shutdown_dispatch();
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

pub struct ThreadSpanGuard {
    // the value of the function pointer will identity the scope uniquely within that process
    // instance
    pub thread_span_desc: &'static ThreadSpanDesc,
    pub _dummy_ptr: PhantomData<*mut u8>, // to mark the object as !Send
}

impl Drop for ThreadSpanGuard {
    fn drop(&mut self) {
        on_end_scope(self.thread_span_desc);
    }
}
