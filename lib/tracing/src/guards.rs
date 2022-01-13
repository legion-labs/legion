use std::{marker::PhantomData, sync::Arc};

use crate::{
    dispatch::{
        flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
        init_thread_stream, on_end_scope, shutdown_dispatch,
    },
    event::EventSink,
    panic_hook::init_panic_hook,
    spans::ThreadSpanMetadata,
};

pub struct TracingSystemGuard {}

impl TracingSystemGuard {
    pub fn new(
        logs_buffer_size: usize,
        metrics_buffer_size: usize,
        threads_buffer_size: usize,
        sink: Arc<dyn EventSink>,
    ) -> anyhow::Result<Self> {
        init_telemetry(
            logs_buffer_size,
            metrics_buffer_size,
            threads_buffer_size,
            sink,
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
    sink: Arc<dyn EventSink>,
) -> anyhow::Result<()> {
    init_event_dispatch(
        logs_buffer_size,
        metrics_buffer_size,
        threads_buffer_size,
        sink,
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

pub struct ThreadSpanGuard {
    // the value of the function pointer will identity the scope uniquely within that process
    // instance
    pub thread_span_desc: &'static ThreadSpanMetadata,
    pub _dummy_ptr: PhantomData<*mut u8>, // to mark the object as !Send
}

impl Drop for ThreadSpanGuard {
    fn drop(&mut self) {
        on_end_scope(self.thread_span_desc);
    }
}
