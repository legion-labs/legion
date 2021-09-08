#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct LogMsgEvent {
    pub level: LogLevel,
    pub msg: &'static str,
}

pub struct LogStream {
    pub events: Vec<LogMsgEvent>,
}

impl LogStream {
    pub fn new(buffer_size: usize) -> Self {
        let mut events = Vec::new();
        events.reserve(buffer_size);
        Self { events }
    }

    pub fn push(&mut self, event: LogMsgEvent) {
        self.events.push(event);
    }

    pub fn is_full(&self) -> bool {
        self.events.capacity() == self.events.len()
    }
}
