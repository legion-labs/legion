use std::sync::Arc;
use std::thread;

use telemetry::*;

struct DebugEventSink {}

impl EventBlockSink for DebugEventSink {
    fn on_log_buffer_full(&self, log_stream: &mut LogStream) {
        println!("log buffer full: {} events", log_stream.events.len());
        // for evt in &log_stream.events {
        //     println!("{:?} {}", evt.level, evt.msg);
        // }
    }
}

fn test_log_str() {
    for _ in 1..5 {
        log_str(LogLevel::Info, "test");
    }
}

fn test_log_thread() {
    let mut threads = Vec::new();
    for _ in 1..5 {
        threads.push(thread::spawn(|| {
            println!("from thread {:?}", thread::current().id());
            for _ in 1..1024 {
                log_str(LogLevel::Info, "test_msg");
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
}

#[test]
fn test_log() {
    let sink: Arc<dyn EventBlockSink> = Arc::new(DebugEventSink {});
    init_event_dispatch(1024, sink).unwrap();
    test_log_str();
    test_log_thread();
}
