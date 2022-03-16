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

pub struct CompositeSink {
    sinks: Vec<BoxedEventSink>,
}

impl CompositeSink {
    pub fn new(sinks: Vec<BoxedEventSink>) -> Self {
        Self { sinks }
    }
}

impl From<Vec<BoxedEventSink>> for CompositeSink {
    fn from(sinks: Vec<BoxedEventSink>) -> Self {
        Self::new(sinks)
    }
}

impl From<Vec<Self>> for BoxedEventSink {
    fn from(sinks: Vec<Self>) -> Self {
        Box::new(CompositeSink::from(sinks))
    }
}

impl EventSink for CompositeSink {
    fn on_startup(&self, process_info: ProcessInfo) {
        if self.sinks.len() == 1 {
            self.sinks[0].on_startup(process_info);
        } else {
            self.sinks
                .iter()
                .for_each(|sink| sink.on_startup(process_info.clone()));
        }
    }

    fn on_shutdown(&self) {
        self.sinks.iter().for_each(|sink| sink.on_shutdown());
    }

    fn on_log_enabled(&self, level: Level, target: &str) -> bool {
        // The log is enabled if _all_ the sinks are enabled
        // If the sinks vec is empty `false` will be returned
        self.sinks
            .iter()
            .all(|sink| sink.on_log_enabled(level, target))
    }

    fn on_log(&self, desc: &LogMetadata, time: i64, args: fmt::Arguments<'_>) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_log(desc, time, args));
    }

    fn on_init_log_stream(&self, log_stream: &LogStream) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_init_log_stream(log_stream));
    }

    fn on_process_log_block(&self, old_event_block: Arc<LogBlock>) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_process_log_block(Arc::clone(&old_event_block)));
    }

    fn on_init_metrics_stream(&self, metrics_stream: &MetricsStream) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_init_metrics_stream(metrics_stream));
    }

    fn on_process_metrics_block(&self, old_event_block: Arc<MetricsBlock>) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_process_metrics_block(Arc::clone(&old_event_block)));
    }

    fn on_init_thread_stream(&self, thread_stream: &ThreadStream) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_init_thread_stream(thread_stream));
    }

    fn on_process_thread_block(&self, old_event_block: Arc<ThreadBlock>) {
        self.sinks
            .iter()
            .for_each(|sink| sink.on_process_thread_block(Arc::clone(&old_event_block)));
    }
}
