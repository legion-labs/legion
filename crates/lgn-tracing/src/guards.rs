use std::{marker::PhantomData, sync::Arc};

use crate::{
    dispatch::{
        flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
        init_thread_stream, on_end_async_scope, on_end_scope, shutdown_dispatch,
    },
    errors::Result,
    event::BoxedEventSink,
    panic_hook::init_panic_hook,
    spans::SpanMetadata,
};

pub struct TracingSystemGuard {}

impl TracingSystemGuard {
    pub fn new(
        logs_buffer_size: usize,
        metrics_buffer_size: usize,
        threads_buffer_size: usize,
        sinks: Arc<Vec<BoxedEventSink>>,
    ) -> Result<Self> {
        init_telemetry(
            logs_buffer_size,
            metrics_buffer_size,
            threads_buffer_size,
            sinks,
        )?;
        Ok(Self {})
    }
}

impl std::ops::Drop for TracingSystemGuard {
    fn drop(&mut self) {
        shutdown_telemetry();
    }
}

pub fn init_telemetry(
    logs_buffer_size: usize,
    metrics_buffer_size: usize,
    threads_buffer_size: usize,
    sinks: Arc<Vec<BoxedEventSink>>,
) -> Result<()> {
    init_event_dispatch(
        logs_buffer_size,
        metrics_buffer_size,
        threads_buffer_size,
        sinks,
    )?;
    init_panic_hook();
    Ok(())
}

pub fn shutdown_telemetry() {
    flush_log_buffer();
    flush_metrics_buffer();
    shutdown_dispatch();
}

pub struct TracingThreadGuard {
    _dummy_ptr: *mut u8,
}

impl TracingThreadGuard {
    pub fn new() -> Self {
        init_thread_stream();
        Self {
            _dummy_ptr: std::ptr::null_mut(),
        }
    }
}

impl std::ops::Drop for TracingThreadGuard {
    fn drop(&mut self) {
        flush_thread_buffer();
    }
}

//not used at the time of writing, but clippy wants it
impl Default for TracingThreadGuard {
    fn default() -> Self {
        Self::new()
    }
}

// sync scope guard
pub struct ThreadSpanGuard {
    pub thread_span_desc: &'static SpanMetadata,
    pub _dummy_ptr: PhantomData<*mut u8>, // to mark the object as !Send
}

impl Drop for ThreadSpanGuard {
    fn drop(&mut self) {
        on_end_scope(self.thread_span_desc);
    }
}

// async scope guard
pub struct AsyncSpanGuard {
    pub span_desc: &'static SpanMetadata,
    pub span_id: u64,
}

impl Drop for AsyncSpanGuard {
    fn drop(&mut self) {
        on_end_async_scope(self.span_id, self.span_desc);
    }
}
