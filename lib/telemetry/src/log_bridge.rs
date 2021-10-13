use crate::{log_string, LogLevel};
use anyhow::Result;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct LogBridge {
    pub opt_app_log: Option<Box<dyn log::Log>>,
}

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
            if let Some(app_log) = &self.opt_app_log {
                app_log.log(record);
            }
        }
    }

    fn flush(&self) {}
}

// setup_log_bridge sets the log crate's logger and forwards all the records into telemetry
pub fn setup_log_bridge(opt_app_log: Option<Box<dyn log::Log>>) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(LogBridge { opt_app_log }))
        .map(|()| log::set_max_level(LevelFilter::Trace))?;
    Ok(())
}
