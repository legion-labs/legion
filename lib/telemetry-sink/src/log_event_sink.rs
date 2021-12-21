use std::sync::Arc;

use lgn_telemetry::{
    log, log::Log, EventSink, LogBlock, LogStream, MetricsBlock, MetricsStream, ProcessInfo,
    ThreadBlock, ThreadStream,
};
use simple_logger::SimpleLogger;

pub struct SimpleLoggerEventSink {
    simple_logger: SimpleLogger,
}

impl SimpleLoggerEventSink {
    pub fn new() -> Self {
        Self {
            simple_logger: SimpleLogger::new().with_utc_timestamps(),
        }
    }
}

impl EventSink for SimpleLoggerEventSink {
    fn on_startup(&self, _: ProcessInfo) {}
    fn on_shutdown(&self) {}

    fn on_log_enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.simple_logger.enabled(metadata)
    }
    fn on_log(&self, record: &log::Record<'_>) {
        self.simple_logger.log(record);
    }
    fn on_init_log_stream(&self, _: &LogStream) {}
    fn on_process_log_block(&self, _: Arc<LogBlock>) {}

    fn on_init_metrics_stream(&self, _: &MetricsStream) {}
    fn on_process_metrics_block(&self, _: Arc<MetricsBlock>) {}

    fn on_init_thread_stream(&self, _: &ThreadStream) {}
    fn on_process_thread_block(&self, _: Arc<ThreadBlock>) {}
}
