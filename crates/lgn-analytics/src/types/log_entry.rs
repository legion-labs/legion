#[derive(Clone, PartialEq)]
pub struct LogEntry {
    pub level: i32,
    pub time_ms: f64,
    pub target: String,
    pub msg: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

#[derive(Clone, PartialEq)]
pub struct ProcessLogReply {
    pub entries: Vec<LogEntry>,
    /// included
    pub begin: u64,
    /// excluded
    pub end: u64,
}

impl From<LogEntry> for crate::api::components::LogEntry {
    fn from(log_entry: LogEntry) -> Self {
        Self {
            level: log_entry.level,
            time_ms: log_entry.time_ms,
            target: log_entry.target,
            msg: log_entry.msg,
        }
    }
}

impl From<crate::api::components::LogEntry> for LogEntry {
    fn from(log_entry: crate::api::components::LogEntry) -> Self {
        Self {
            level: log_entry.level,
            time_ms: log_entry.time_ms,
            target: log_entry.target,
            msg: log_entry.msg,
        }
    }
}
