use crate::{
    flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
    init_thread_stream,
    panic_hook::{init_ctrlc_hook, init_panic_hook},
    setup_log_bridge, shutdown_event_dispatch, EventBlockSink, GRPCEventSink, NullEventSink,
};
use std::sync::Arc;

pub struct TelemetrySystemGuard {}

impl TelemetrySystemGuard {
    pub fn new(app_log: Option<Box<dyn log::Log>>) -> Self {
        init_telemetry(app_log);
        Self {}
    }
}

impl std::ops::Drop for TelemetrySystemGuard {
    fn drop(&mut self) {
        shutdown_telemetry();
    }
}

pub fn init_telemetry(app_log: Option<Box<dyn log::Log>>) {
    let make_sink: Box<dyn FnOnce() -> Arc<dyn EventBlockSink>> =
        match std::env::var("LEGION_TELEMETRY_URL") {
            Ok(url) => Box::new(move || Arc::new(GRPCEventSink::new(&url))),
            Err(_no_url_in_env) => Box::new(|| Arc::new(NullEventSink {})),
        };
    if let Err(_e) = init_event_dispatch(10 * 1024 * 1024, 5 * 1024 * 1024, 1024 * 1024, make_sink)
    {
        return;
    }
    init_panic_hook();
    init_ctrlc_hook();
    setup_log_bridge(app_log).unwrap();
}

pub fn shutdown_telemetry() {
    flush_log_buffer();
    flush_metrics_buffer();
    shutdown_event_dispatch();
}

pub struct TelemetryThreadGuard {}

impl TelemetryThreadGuard {
    pub fn new() -> Self {
        init_thread_stream();
        Self {}
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
