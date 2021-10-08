use std::collections::HashSet;

use crate::{compress, DualTime, EncodedBlock, LogDynMsgEvent, LogMsgEvent, StreamBlock};
use anyhow::Result;
use transit::{
    declare_queue_struct, read_pod, IterableQueue, QueueIterator, Reflect, StaticString,
    UserDefinedType,
};

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent, LogDynMsgEvent> {}
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
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = LogDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.events.iter() {
            match x {
                LogMsgQueueAny::LogMsgEvent(evt) => {
                    if recorded_deps.insert(evt.msg as u64) {
                        deps.push(StaticString {
                            len: evt.msg_len,
                            ptr: evt.msg,
                        });
                    }
                }
                LogMsgQueueAny::LogDynMsgEvent(_evt) => {}
            }
        }

        let payload = legion_telemetry_proto::ingestion::BlockPayload {
            dependencies: compress(deps.as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok(EncodedBlock {
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
        })
    }
}
