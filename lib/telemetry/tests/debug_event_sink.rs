use std::sync::{Arc, Mutex};

use lgn_telemetry::{
    log, EventSink, Level, LogBlock, LogMsgQueueAny, LogStream, MetricsBlock, MetricsMsgQueueAny,
    MetricsStream, ProcessInfo, ThreadBlock, ThreadEventQueueAny, ThreadStream,
};
use lgn_transit::HeterogeneousQueue;
use LogMsgQueueAny::{LogDynMsgEvent, LogMsgEvent};
use MetricsMsgQueueAny::{FloatMetricEvent, IntegerMetricEvent};
use ThreadEventQueueAny::{BeginScopeEvent, EndScopeEvent};

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

    fn on_log_enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        *self.0.lock().unwrap() = Some(State::LogEnabled(metadata.level()));
        true
    }

    fn on_log(&self, record: &log::Record<'_>) {
        *self.0.lock().unwrap() = Some(State::Log(record.args().to_string()));
    }

    fn on_init_log_stream(&self, _: &LogStream) {
        *self.0.lock().unwrap() = Some(State::InitLogStream);
    }

    fn on_process_log_block(&self, log_block: std::sync::Arc<LogBlock>) {
        for event in log_block.events.iter() {
            match event {
                LogMsgEvent(_evt) => {}
                LogDynMsgEvent(_evt) => {}
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
                IntegerMetricEvent(_evt) => {}
                FloatMetricEvent(_evt) => {}
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
                BeginScopeEvent(_evt) => {}
                EndScopeEvent(_evt) => {}
            }
        }
        *self.0.lock().unwrap() = Some(State::ProcessThreadBlock(thread_block.events.nb_objects()));
    }
}

pub fn expect(state: &SharedState, expected: &Option<State>) {
    let state = state.lock().unwrap().take();
    assert_eq!(state, *expected);
}
