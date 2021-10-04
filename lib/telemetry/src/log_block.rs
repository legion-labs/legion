use crate::*;
use transit::*;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info = 1,
    Warning = 2,
    Error = 3,
}

#[derive(Debug, TransitReflect)]
pub struct LogMsgEvent {
    pub level: u8,
    pub msg_len: u32,
    pub msg: *const u8,
}

impl Serialize for LogMsgEvent {}

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent> {}
);

declare_queue_struct!(
    struct LogDepsQueue<StaticString> {}
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

        let mut deps = LogDepsQueue::new(1024 * 1024);
        for x in self.events.iter() {
            match x {
                LogMsgQueueAny::LogMsgEvent(evt) => {
                    deps.push(StaticString {
                        len: evt.msg_len,
                        ptr: evt.msg,
                    });
                }
            }
        }

        let payload = telemetry_ingestion_proto::BlockPayload {
            dependencies: deps.into_bytes(),
            objects: self.events.as_bytes().to_vec(),
        };

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
            payload: Some(payload),
        }
    }
}
