//! A generic [`EventSink`] that accepts an [`broadcast::Sender<TraceEvent>`]
//! that will be used to send [`TraceEvent`] messages.

use std::{fmt, sync::Arc};

use lgn_tracing::{
    event::EventSink,
    logs::{LogBlock, LogMetadata, LogStream},
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadStream},
    Level, ProcessInfo,
};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum TraceEvent {
    Startup(ProcessInfo),
    Shutdown,
    Message {
        target: String,
        message: String,
        level: Level,
        time: i64,
    },
    InitLogStream,
    ProcessLogBlock(Arc<LogBlock>),
    InitMetricsStream,
    ProcessMetricsBlock(Arc<MetricsBlock>),
    InitThreadStream,
    ProcessThreadBlock(Arc<ThreadBlock>),
}

pub struct BroadcastSink {
    sender: broadcast::Sender<TraceEvent>,
}

impl BroadcastSink {
    pub fn new(sender: broadcast::Sender<TraceEvent>) -> Self {
        Self { sender }
    }

    fn send(&self, trace_event: TraceEvent) {
        let sender = self.sender.clone();

        let _send_result = sender.send(trace_event);
    }
}

impl EventSink for BroadcastSink {
    fn on_startup(&self, process_info: ProcessInfo) {
        self.send(TraceEvent::Startup(process_info));
    }

    fn on_shutdown(&self) {
        self.send(TraceEvent::Shutdown);
    }

    fn on_log_enabled(&self, _metadata: &LogMetadata) -> bool {
        // TODO: Do editor specific filtering here
        true
    }

    fn on_log(&self, metadata: &LogMetadata, time: i64, args: fmt::Arguments<'_>) {
        self.send(TraceEvent::Message {
            level: metadata.level,
            time,
            target: metadata.target.to_string(),
            message: format!("{}", args),
        });
    }

    fn on_init_log_stream(&self, _log_stream: &LogStream) {
        self.send(TraceEvent::InitLogStream);
    }

    fn on_process_log_block(&self, log_block: Arc<LogBlock>) {
        self.send(TraceEvent::ProcessLogBlock(log_block));
    }

    fn on_init_metrics_stream(&self, _metric_stream: &MetricsStream) {
        self.send(TraceEvent::InitMetricsStream);
    }

    fn on_process_metrics_block(&self, metrics_block: Arc<MetricsBlock>) {
        self.send(TraceEvent::ProcessMetricsBlock(metrics_block));
    }

    fn on_init_thread_stream(&self, _thread_stream: &ThreadStream) {
        self.send(TraceEvent::InitThreadStream);
    }

    fn on_process_thread_block(&self, thread_block: Arc<ThreadBlock>) {
        self.send(TraceEvent::ProcessThreadBlock(thread_block));
    }

    fn is_busy(&self) -> bool {
        false
    }
}
