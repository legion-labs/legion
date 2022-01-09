use std::collections::HashSet;

use lgn_tracing_transit::prelude::*;

use crate::event_block::{EventBlock, ExtractDeps};
use crate::{EventStream, LogDynMsgEvent, LogMsgEvent};

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent, LogDynMsgEvent> {}
);

declare_queue_struct!(
    struct LogDepsQueue<StaticString> {}
);

impl ExtractDeps for LogMsgQueue {
    type DepsQueue = LogDepsQueue;

    fn extract(&self) -> Self::DepsQueue {
        let mut deps = LogDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.iter() {
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
        deps
    }
}

pub type LogBlock = EventBlock<LogMsgQueue>;
pub type LogStream = EventStream<LogBlock>;
