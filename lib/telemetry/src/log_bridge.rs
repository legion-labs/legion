use crate::{log_string, LogLevel};
use anyhow::Result;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct LogBridge;

impl log::Log for LogBridge {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        let metadata = record.metadata();
        if self.enabled(metadata) {
            //todo: optimize this string format either by moving it to the telemetry publishing thread
            // or by sending the format args on the wire
            log_string(
                LogLevel::from(record.level()),
                format!("target={} {}", metadata.target(), record.args()),
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: LogBridge = LogBridge;

pub fn setup_log_bridge() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))?;
    Ok(())
}
