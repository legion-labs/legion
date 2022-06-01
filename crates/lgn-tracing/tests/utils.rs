use std::{
    fmt,
    sync::{Arc, Mutex},
};

use lgn_tracing::{
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

    fn on_log_enabled(&self, metadata: &LogMetadata) -> bool {
        *self.0.lock().unwrap() = Some(State::LogEnabled(metadata.level));
        true
    }

    fn on_log(&self, _desc: &LogMetadata, _time: i64, args: fmt::Arguments<'_>) {
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
                ThreadEventQueueAny::BeginThreadNamedSpanEvent(_evt) => {}
                ThreadEventQueueAny::EndThreadNamedSpanEvent(_evt) => {}
                ThreadEventQueueAny::BeginAsyncSpanEvent(_evt) => {}
                ThreadEventQueueAny::EndAsyncSpanEvent(_evt) => {}
                ThreadEventQueueAny::BeginAsyncNamedSpanEvent(_evt) => {}
                ThreadEventQueueAny::EndAsyncNamedSpanEvent(_evt) => {}
            }
        }
        *self.0.lock().unwrap() = Some(State::ProcessThreadBlock(thread_block.events.nb_objects()));
    }

    fn is_busy(&self) -> bool {
        false
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
