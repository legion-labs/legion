use std::sync::{Arc, Mutex};
use std::thread;

mod utils;
use lgn_tracing::dispatch::{
    flush_log_buffer, flush_metrics_buffer, flush_thread_buffer, init_event_dispatch,
    init_thread_stream, process_id,
};
use lgn_tracing::{info, metric_float, metric_int, set_max_level, trace_scope, LevelFilter};
use lgn_tracing_proc_macros::{log_function, trace_function};
use utils::{DebugEventSink, LogDispatch, SharedState, State};

fn test_log_str(state: &SharedState) {
    for x in 0..5 {
        info!("test");
        expect_state!(state, Some(State::Log(String::from("test"))));
        info!("test {}", x);
        expect_state!(state, Some(State::Log(format!("test {}", x))));
    }
    flush_log_buffer();
    expect_state!(state, Some(State::ProcessLogBlock(10)));
}

fn test_log_interop_str(state: &SharedState) {
    for x in 0..5 {
        log::info!("test");
        expect_state!(state, Some(State::Log(String::from("test"))));
        log::info!("test {}", x);
        expect_state!(state, Some(State::Log(format!("test {}", x))));
    }
    flush_log_buffer();
    expect_state!(state, Some(State::ProcessLogBlock(10)));
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
        .map_or_else(|| "unknown".to_owned(), |b| b.as_str().to_owned());

    println!("CPU brand: {:?}", &cpu_brand);

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

fn test_thread_spans(state: SharedState) {
    println!("TSC frequency: {}", get_tsc_frequency().unwrap_or_default());
    let mut threads = Vec::new();
    for _ in 0..5 {
        threads.push(thread::spawn(move || {
            init_thread_stream();
            for _ in 0..1024 {
                trace_scope!("test");
            }
            flush_thread_buffer();
        }));
    }
    for t in threads {
        t.join().unwrap();
    }

    init_thread_stream();
    for _ in 0..1024 {
        trace_scope!("test");
    }
    flush_thread_buffer();
    expect_state!(state, Some(State::ProcessThreadBlock(2048)));
}

fn test_metrics(state: &SharedState) {
    metric_int!("ticks", "Frame Time", 1000);
    metric_float!("ticks", "Frame Time", 1.0);
    flush_metrics_buffer();
    expect_state!(state, Some(State::ProcessMetricsBlock(2)));
}

#[trace_function]
fn trace_func() {}

#[trace_function("foo")]
fn trace_func_named() {}

#[log_function]
fn log_func() {}

fn test_proc_macros(state: &SharedState) {
    trace_func();
    trace_func_named();
    flush_thread_buffer();
    expect_state!(&state.clone(), Some(utils::State::ProcessThreadBlock(4)));

    log_func();
    expect_state!(state, Some(State::Log(String::from("log_func"))));
}

#[test]
fn test_log() {
    static LOG_DISPATCHER: LogDispatch = LogDispatch;
    log::set_logger(&LOG_DISPATCHER).unwrap();

    let state = Arc::new(Mutex::new(None));
    init_event_dispatch(
        10 * 1024,
        64 * 1024,
        12 * 1024,
        Arc::new(DebugEventSink::new(state.clone())),
    )
    .unwrap();
    set_max_level(LevelFilter::Trace);
    log::set_max_level(log::LevelFilter::Trace);
    assert!(process_id().is_some());
    test_log_str(&state);
    test_log_interop_str(&state);
    test_thread_spans(state.clone());
    test_proc_macros(&state);
    test_metrics(&state);
}
