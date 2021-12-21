use std::sync::{Arc, Mutex};
use std::thread;

use lgn_telemetry::*;
mod debug_event_sink;
use debug_event_sink::{expect, DebugEventSink, SharedState, State};

fn test_log_str(state: &SharedState) {
    for x in 1..5 {
        info!("test");
        expect(state, Some(State::Log(String::from("test"))));
        info!("test {}", x);
        expect(state, Some(State::Log(format!("test {}", x))));
    }
    flush_log_buffer();
    expect(state, Some(State::ProcessLogBlock(7)));
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

fn test_log_thread(state: &SharedState) {
    println!("TSC frequency: {}", get_tsc_frequency().unwrap_or_default());
    let mut threads = Vec::new();
    for _ in 1..5 {
        threads.push(thread::spawn(|| {
            init_thread_stream();
            for _ in 1..1024 {
                trace_scope!();
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    flush_log_buffer();
    expect(state, Some(State::ProcessThreadBlock(723)));
}

fn test_metrics(state: &SharedState) {
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Frame Time",
        unit: "ticks",
    };
    dbg!(&FRAME_TIME_METRIC);
    record_int_metric(&FRAME_TIME_METRIC, 1000);
    record_float_metric(&FRAME_TIME_METRIC, 1.0);
    flush_metrics_buffer();
    expect(state, Some(State::ProcessMetricsBlock(2)));
}

#[test]
fn test_log() {
    let state = Arc::new(Mutex::new(None));
    init_event_dispatch(
        1024 * 10,
        1024 * 12,
        1024 * 12,
        Arc::new(DebugEventSink::new(state.clone())),
    )
    .unwrap();
    set_max_log_level(LevelFilter::Trace);
    assert!(get_process_id().is_some());
    test_log_str(&state);
    test_log_thread(&state);
    test_metrics(&state);
}
