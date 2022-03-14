//! A generic [`EventSink`] that accepts an [`mpsc::Sender<Log>`]
//! that will be used to send [`Log`] messages.

use core::fmt;
use std::sync::Arc;

use lgn_tracing::{
    event::EventSink,
    logs::{LogBlock, LogMetadata, LogStream},
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadStream},
    Level, ProcessInfo,
};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Log {
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

pub struct ChannelSink {
    sender: mpsc::UnboundedSender<Log>,
}

impl ChannelSink {
    pub fn new(sender: mpsc::UnboundedSender<Log>) -> Self {
        Self { sender }
    }

    fn send(&self, log: Log) {
        let sender = self.sender.clone();

        let _send_result = sender.send(log);
    }
}

impl EventSink for ChannelSink {
    fn on_startup(&self, process_info: ProcessInfo) {
        self.send(Log::Startup(process_info));
    }

    fn on_shutdown(&self) {
        self.send(Log::Shutdown);
    }

    fn on_log_enabled(&self, _level: Level, _: &str) -> bool {
        true
    }

    fn on_log(&self, desc: &LogMetadata, time: i64, args: fmt::Arguments<'_>) {
        self.send(Log::Message {
            level: desc.level,
            time,
            target: desc.target.to_string(),
            message: format!("{}", args),
        });
    }

    fn on_init_log_stream(&self, _log_stream: &LogStream) {
        self.send(Log::InitLogStream);
    }

    fn on_process_log_block(&self, log_block: Arc<LogBlock>) {
        self.send(Log::ProcessLogBlock(log_block));
    }

    fn on_init_metrics_stream(&self, _metric_stream: &MetricsStream) {
        self.send(Log::InitMetricsStream);
    }

    fn on_process_metrics_block(&self, metrics_block: Arc<MetricsBlock>) {
        self.send(Log::ProcessMetricsBlock(metrics_block));
    }

    fn on_init_thread_stream(&self, _thread_stream: &ThreadStream) {
        self.send(Log::InitThreadStream);
    }

    fn on_process_thread_block(&self, thread_block: Arc<ThreadBlock>) {
        self.send(Log::ProcessThreadBlock(thread_block));
    }
}
