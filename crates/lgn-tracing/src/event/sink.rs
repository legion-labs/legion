use std::{fmt, sync::Arc};

use crate::{
    logs::{LogBlock, LogMetadata, LogStream},
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadStream},
    Level, ProcessInfo,
};

pub type BoxedEventSink = Box<dyn EventSink>;

pub trait EventSink {
    fn on_startup(&self, process_info: ProcessInfo);
    fn on_shutdown(&self);

    fn on_log_enabled(&self, level: Level, target: &str) -> bool;
    fn on_log(&self, desc: &LogMetadata, time: i64, args: fmt::Arguments<'_>);
    fn on_init_log_stream(&self, log_stream: &LogStream);
    fn on_process_log_block(&self, log_block: Arc<LogBlock>);

    fn on_init_metrics_stream(&self, metrics_stream: &MetricsStream);
    fn on_process_metrics_block(&self, metrics_block: Arc<MetricsBlock>);

    fn on_init_thread_stream(&self, thread_stream: &ThreadStream);
    fn on_process_thread_block(&self, thread_block: Arc<ThreadBlock>);
}

pub struct NullEventSink {}

impl EventSink for NullEventSink {
    fn on_startup(&self, _: ProcessInfo) {}
    fn on_shutdown(&self) {}

    fn on_log_enabled(&self, _: Level, _: &str) -> bool {
        false
    }
    fn on_log(&self, _: &LogMetadata, _: i64, _: fmt::Arguments<'_>) {}
    fn on_init_log_stream(&self, _: &LogStream) {}
    fn on_process_log_block(&self, _: Arc<LogBlock>) {}

    fn on_init_metrics_stream(&self, _: &MetricsStream) {}
    fn on_process_metrics_block(&self, _: Arc<MetricsBlock>) {}

    fn on_init_thread_stream(&self, _: &ThreadStream) {}
    fn on_process_thread_block(&self, _: Arc<ThreadBlock>) {}
}
