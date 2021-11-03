use std::collections::HashSet;

use anyhow::Result;
use legion_transit::prelude::*;

use crate::{
    compress, event_block::EventBlock, EncodedBlock, EventStream, LogDynMsgEvent, LogMsgEvent,
    StreamBlock,
};

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent, LogDynMsgEvent> {}
);

declare_queue_struct!(
    struct LogDepsQueue<StaticString> {}
);

pub type LogBlock = EventBlock<LogMsgQueue>;

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

        let payload = legion_telemetry_proto::telemetry::BlockPayload {
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

pub type LogStream = EventStream<LogBlock, LogDepsQueue>;
