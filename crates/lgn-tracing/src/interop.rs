use std::sync::atomic::AtomicU32;

use crate::{
    dispatch::{flush_log_buffer, log_enabled, log_interop},
    logs::{LogMetadata, FILTER_LEVEL_UNSET_VALUE},
};

pub struct LogDispatch;

impl log::Log for LogDispatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let log_metadata = LogMetadata {
            level: metadata.level().into(),
            level_filter: AtomicU32::new(0),
            fmt_str: "",
            target: "unknown",
            module_path: "unknown",
            file: "unknown",
            line: 0,
        };

        log_enabled(&log_metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        let log_metadata = LogMetadata {
            level: record.level().into(),
            level_filter: AtomicU32::new(FILTER_LEVEL_UNSET_VALUE),
            fmt_str: record.args().as_str().unwrap_or_default(),
            target: record.module_path_static().unwrap_or("unknown"),
            module_path: record.module_path_static().unwrap_or("unknown"),
            file: record.file_static().unwrap_or("unknown"),
            line: record.line().unwrap_or_default(),
        };

        log_interop(&log_metadata, *record.args());
    }

    fn flush(&self) {
        flush_log_buffer();
    }
}
