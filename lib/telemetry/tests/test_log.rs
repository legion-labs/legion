use std::sync::Arc;

use telemetry::*;

struct DebugEventSink {}

impl EventBlockSink for DebugEventSink {
    fn on_log_buffer_full(&self, log_stream: &mut LogStream) {
        for evt in &log_stream.events {
            println!("{:?} {}", evt.level, evt.msg);
        }
    }
}

#[test]
fn test_log_str() {
    let sink: Arc<dyn EventBlockSink> = Arc::new(DebugEventSink {});
    init_event_dispatch(4, sink).unwrap();
    for _ in 1..5 {
        log_str(LogLevel::Info, "test");
    }
}
