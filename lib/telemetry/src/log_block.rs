use crate::*;
use transit::*;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct LogMsgEvent {
    pub level: LogLevel,
    pub msg: &'static str,
}

impl Serialize for LogMsgEvent {}

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent> {}
);

#[derive(Debug)]
pub struct LogBlock {
    pub stream_id: String,
    pub begin: DualTime,
    pub events: LogMsgQueue,
    pub end: Option<DualTime>,
}

impl LogBlock {
    pub fn new(buffer_size: usize, stream_id: String) -> Self {
        let events = LogMsgQueue::new(buffer_size);
        Self {
            stream_id,
            begin: DualTime::now(),
            events,
            end: None,
        }
    }

    pub fn close(&mut self) {
        self.end = Some(DualTime::now());
    }
}

impl StreamBlock for LogBlock {
    fn encode(&self) -> EncodedBlock {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();
        EncodedBlock {
            stream_id: self.stream_id.clone(),
            block_id,
            begin_time: self
                .begin
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            begin_ticks: self.begin.ticks,
            end_time: end
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            end_ticks: end.ticks,
            payload: None,
        }
    }
}
