use std::{fmt, sync::Arc};

use lgn_tracing::{
    event::{BoxedEventSink, EventSink},
    interop::LogDispatch,
    logs::{LogBlock, LogMetadata, LogStream},
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadStream},
    LevelFilter, ProcessInfo,
};

pub struct CompositeSink {
    sinks: Vec<(LevelFilter, BoxedEventSink)>,
    target_level_filters: Vec<(String, LevelFilter)>,
}

impl CompositeSink {
    pub fn new(
        sinks: Vec<(LevelFilter, BoxedEventSink)>,
        target_max_level: Vec<(String, LevelFilter)>,
        max_level_override: Option<LevelFilter>,
        interop_max_level_override: Option<LevelFilter>,
    ) -> Self {
        if let Some(max_level) = max_level_override {
            lgn_tracing::set_max_level(max_level);
        } else {
            let mut max_level = LevelFilter::Off;
            for (_, level_filter) in &target_max_level {
                max_level = max_level.max(*level_filter);
            }
            for (level_filter, _) in &sinks {
                max_level = max_level.max(*level_filter);
            }
            lgn_tracing::set_max_level(max_level);
        }
        let interop_max_level = interop_max_level_override
            .unwrap_or_else(lgn_tracing::max_level)
            .into();
        log::set_max_level(interop_max_level);

        static LOG_DISPATCHER: LogDispatch = LogDispatch;

        if log::set_logger(&LOG_DISPATCHER).is_err() {
            println!("Could not set log crate dispatcher");
            log::set_max_level(log::LevelFilter::Off);
        }

        let mut target_max_level = target_max_level;
        target_max_level.sort_by_key(|(name, _)| name.len().wrapping_neg());

        Self {
            sinks,
            target_level_filters: target_max_level,
        }
    }

    fn target_max_level(&self, metadata: &LogMetadata) -> Option<LevelFilter> {
        const GENERATION: u16 = 1;
        // At this point we would have already tested the max level on the macro
        match metadata.level_filter(GENERATION) {
            lgn_tracing::logs::FilterState::Outdated => {
                let level_filter =
                    Self::find_max_match(metadata.target, &self.target_level_filters);
                metadata.set_level_filter(GENERATION, level_filter);
                level_filter
            }
            lgn_tracing::logs::FilterState::NotSet => None,
            lgn_tracing::logs::FilterState::Set(level_filter) => Some(level_filter),
        }
    }

    /// This needs to be optimized
    fn find_max_match(
        target: &str,
        level_filters: &[(String, LevelFilter)],
    ) -> Option<LevelFilter> {
        for &(ref t, ref l) in level_filters.iter() {
            if target.starts_with(t) {
                return Some(*l);
            }
        }
        None
    }
}

impl EventSink for CompositeSink {
    fn on_startup(&self, process_info: ProcessInfo) {
        if self.sinks.len() == 1 {
            self.sinks[0].1.on_startup(process_info);
        } else {
            self.sinks
                .iter()
                .for_each(|(_, sink)| sink.on_startup(process_info.clone()));
        }
    }

    fn on_shutdown(&self) {
        self.sinks.iter().for_each(|(_, sink)| sink.on_shutdown());
    }

    fn on_log_enabled(&self, metadata: &LogMetadata) -> bool {
        // The log is enabled if any of the sinks are enabled
        // If the sinks vec is empty `false` will be returned
        let target_max_level = self.target_max_level(metadata);
        self.sinks.iter().any(|(max_level, sink)| {
            metadata.level <= target_max_level.unwrap_or(*max_level)
                && sink.on_log_enabled(metadata)
        })
    }

    fn on_log(&self, metadata: &LogMetadata, time: i64, args: fmt::Arguments<'_>) {
        let target_max_level = self.target_max_level(metadata);
        self.sinks.iter().for_each(|(max_level, sink)| {
            if metadata.level <= target_max_level.unwrap_or(*max_level)
                && sink.on_log_enabled(metadata)
            {
                sink.on_log(metadata, time, args);
            }
        });
    }

    fn on_init_log_stream(&self, log_stream: &LogStream) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_init_log_stream(log_stream));
    }

    fn on_process_log_block(&self, old_event_block: Arc<LogBlock>) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_process_log_block(old_event_block.clone()));
    }

    fn on_init_metrics_stream(&self, metrics_stream: &MetricsStream) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_init_metrics_stream(metrics_stream));
    }

    fn on_process_metrics_block(&self, old_event_block: Arc<MetricsBlock>) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_process_metrics_block(old_event_block.clone()));
    }

    fn on_init_thread_stream(&self, thread_stream: &ThreadStream) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_init_thread_stream(thread_stream));
    }

    fn on_process_thread_block(&self, old_event_block: Arc<ThreadBlock>) {
        self.sinks
            .iter()
            .for_each(|(_, sink)| sink.on_process_thread_block(old_event_block.clone()));
    }

    fn is_busy(&self) -> bool {
        for (_, sink) in &self.sinks {
            if sink.is_busy() {
                return true;
            }
        }
        false
    }
}
