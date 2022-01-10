use std::{
    fmt,
    sync::{atomic::AtomicU32, Arc, Mutex},
};

use lgn_tracing::{
    dispatch::{flush_log_buffer, log_enabled, log_interop},
    event::EventSink,
    logs::{LogBlock, LogMetadata, LogMsgQueueAny, LogStream},
    metrics::{MetricsBlock, MetricsMsgQueueAny, MetricsStream},
    spans::{ThreadBlock, ThreadEventQueueAny, ThreadStream},
    Level, ProcessInfo,
};
use lgn_tracing_transit::HeterogeneousQueue;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Startup(bool),
    Shutdown,
    LogEnabled(Level),
    Log(String),
    InitLogStream,
    ProcessLogBlock(usize),
    InitMetricsStream,
    ProcessMetricsBlock(usize),
    InitThreadStream,
    ProcessThreadBlock(usize),
}

pub type SharedState = Arc<Mutex<Option<State>>>;
pub struct DebugEventSink(SharedState);

impl DebugEventSink {
    pub fn new(state: SharedState) -> Self {
        Self(state)
    }
}

impl EventSink for DebugEventSink {
    fn on_startup(&self, process_info: ProcessInfo) {
        *self.0.lock().unwrap() = Some(State::Startup(!process_info.process_id.is_empty()));
    }

    fn on_shutdown(&self) {
        *self.0.lock().unwrap() = Some(State::Shutdown);
    }

    fn on_log_enabled(&self, level: Level, _: &str) -> bool {
        *self.0.lock().unwrap() = Some(State::LogEnabled(level));
        true
    }

    fn on_log(&self, _desc: &LogMetadata, _time: i64, args: &fmt::Arguments<'_>) {
        *self.0.lock().unwrap() = Some(State::Log(args.to_string()));
    }

    fn on_init_log_stream(&self, _: &LogStream) {
        *self.0.lock().unwrap() = Some(State::InitLogStream);
    }

    fn on_process_log_block(&self, log_block: std::sync::Arc<LogBlock>) {
        for event in log_block.events.iter() {
            match event {
                LogMsgQueueAny::LogStaticStrEvent(_evt) => {}
                LogMsgQueueAny::LogStringEvent(_evt) => {}
                LogMsgQueueAny::LogStaticStrInteropEvent(_evt) => {}
                LogMsgQueueAny::LogStringInteropEvent(_evt) => {}
            }
        }
        *self.0.lock().unwrap() = Some(State::ProcessLogBlock(log_block.events.nb_objects()));
    }

    fn on_init_metrics_stream(&self, _: &MetricsStream) {
        *self.0.lock().unwrap() = Some(State::InitMetricsStream);
    }

    fn on_process_metrics_block(&self, metrics_block: std::sync::Arc<MetricsBlock>) {
        for event in metrics_block.events.iter() {
            match event {
                MetricsMsgQueueAny::IntegerMetricEvent(_evt) => {}
                MetricsMsgQueueAny::FloatMetricEvent(_evt) => {}
            }
        }
        *self.0.lock().unwrap() = Some(State::ProcessMetricsBlock(
            metrics_block.events.nb_objects(),
        ));
    }

    fn on_init_thread_stream(&self, _: &ThreadStream) {
        *self.0.lock().unwrap() = Some(State::InitThreadStream);
    }

    fn on_process_thread_block(&self, thread_block: std::sync::Arc<ThreadBlock>) {
        for event in thread_block.events.iter() {
            match event {
                ThreadEventQueueAny::BeginThreadSpanEvent(_evt) => {}
                ThreadEventQueueAny::EndThreadSpanEvent(_evt) => {}
            }
        }
        *self.0.lock().unwrap() = Some(State::ProcessThreadBlock(thread_block.events.nb_objects()));
    }
}

pub struct LogDispatch;

impl log::Log for LogDispatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let level = match metadata.level() {
            log::Level::Error => Level::Error,
            log::Level::Warn => Level::Warn,
            log::Level::Info => Level::Info,
            log::Level::Debug => Level::Debug,
            log::Level::Trace => Level::Trace,
        };
        log_enabled(metadata.target(), level)
    }

    fn log(&self, record: &log::Record<'_>) {
        let level = match record.level() {
            log::Level::Error => Level::Error,
            log::Level::Warn => Level::Warn,
            log::Level::Info => Level::Info,
            log::Level::Debug => Level::Debug,
            log::Level::Trace => Level::Trace,
        };
        let log_desc = LogMetadata {
            level,
            level_filter: AtomicU32::new(0),
            fmt_str: record.args().as_str().unwrap_or(""),
            target: record.module_path_static().unwrap_or("unknown"),
            module_path: record.module_path_static().unwrap_or("unknown"),
            file: record.file_static().unwrap_or("unknown"),
            line: record.line().unwrap_or(0),
        };
        log_interop(&log_desc, record.args());
    }
    fn flush(&self) {
        flush_log_buffer();
    }
}

#[macro_export]
macro_rules! expect_state {
    ($state:expr, $expected:expr) => {{
        let state = $state.lock().unwrap().take();
        assert_eq!(state, $expected)
    }};
    () => {};
}
