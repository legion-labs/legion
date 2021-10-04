use crate::*;
use std::sync::Arc;

pub struct TelemetrySystemGuard {}

impl TelemetrySystemGuard {
    pub fn new() -> Self {
        init_telemetry();
        Self {}
    }
}

//not used at the time of writing, but clippy wants it
impl Default for TelemetrySystemGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Drop for TelemetrySystemGuard {
    fn drop(&mut self) {
        shutdown_telemetry();
    }
}

pub fn init_telemetry() {
    let sink: Arc<dyn EventBlockSink> = match std::env::var("LEGION_TELEMETRY_URL") {
        Ok(url) => Arc::new(GRPCEventSink::new(&url)),
        Err(_no_url_in_env) => Arc::new(NullEventSink {}),
    };
    init_event_dispatch(1024, 1024 * 1024, sink).unwrap();
}

pub fn shutdown_telemetry() {
    flush_log_buffer();
    shutdown_event_dispatch();
}
