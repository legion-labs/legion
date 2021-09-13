use std::sync::Arc;
use std::thread;

use telemetry::*;

struct DebugEventSink {}

impl EventBlockSink for DebugEventSink {
    fn on_log_buffer_full(&self, log_block: &LogMsgBlock) {
        println!("log buffer full: {} events", log_block.events.len());
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

fn type_name_of<T>(_: &T) -> &'static str {
    //until type_name_of_val is out of nightly-only
    std::any::type_name::<T>()
}

fn get_tsc_frequency() -> Result<u64, String> {
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();
    let cpu_brand = cpuid
        .get_processor_brand_string()
        .map(|b| b.as_str().to_owned())
        .unwrap_or_else(|| "unknown".to_owned());

    match cpuid.get_tsc_info() {
        Some(tsc_info) => match tsc_info.tsc_frequency() {
            Some(frequency) => Ok(frequency),
            None => Err(format!(
                "tsc frequency unavailable for processor {}",
                cpu_brand
            )),
        },
        None => Err(format!("tsc info unavailable for processor {}", cpu_brand)),
    }
}

fn test_log_thread() {
    println!("event frequency: {}", get_tsc_frequency().unwrap());
    trace_scope!();
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
