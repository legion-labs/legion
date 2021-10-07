use std::sync::Arc;
use std::thread;

use telemetry::LogMsgQueueAny::*;
use telemetry::ThreadEventQueueAny::*;
use telemetry::*;

struct DebugEventSink {}

impl EventBlockSink for DebugEventSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent) {
        match event {
            TelemetrySinkEvent::OnInitProcess(_process_info) => {}
            TelemetrySinkEvent::OnInitStream(_stream_info) => {}
            TelemetrySinkEvent::OnLogBufferFull(log_buffer) => {
                for event in log_buffer.events.iter() {
                    match event {
                        LogMsgEvent(_evt) => {}
                        LogDynMsgEvent(_evt) => {}
                    }
                }
            }
            TelemetrySinkEvent::OnThreadBufferFull(thread_buffer) => {
                for event in thread_buffer.events.iter() {
                    match event {
                        BeginScopeEvent(_evt) => {}
                        EndScopeEvent(_evt) => {}
                    }
                }
            }
            TelemetrySinkEvent::OnShutdown => todo!(),
        }
    }
}

fn test_log_str() {
    for _ in 1..5 {
        log_str(LogLevel::Info, "test");
    }
}

fn get_tsc_frequency() -> Result<u64, String> {
    // does not work in WSL
    // more about the tsc frequency
    //  https://stackoverflow.com/questions/35123379/getting-tsc-rate-from-x86-kernel
    //  https://blog.trailofbits.com/2019/10/03/tsc-frequency-for-all-better-profiling-and-benchmarking/
    //  https://stackoverflow.com/questions/51919219/determine-tsc-frequency-on-linux
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();
    let cpu_brand = cpuid
        .get_processor_brand_string()
        .map(|b| b.as_str().to_owned())
        .unwrap_or_else(|| "unknown".to_owned());

    dbg!(&cpu_brand);

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
    println!("TSC frequency: {}", get_tsc_frequency().unwrap_or_default());
    let mut threads = Vec::new();
    for _ in 1..5 {
        threads.push(thread::spawn(|| {
            init_thread_stream();
            for _ in 1..1024 {
                trace_scope!();
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
    init_event_dispatch(1024 * 10, 1024 * 12, sink).unwrap();
    test_log_str();
    test_log_thread();
}
