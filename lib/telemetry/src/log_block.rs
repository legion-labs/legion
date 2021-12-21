use anyhow::Result;
use lgn_transit::prelude::*;

use crate::event_block::EventBlock;
use crate::{EventStream, LogDynMsgEvent, LogMsgEvent};

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent, LogDynMsgEvent> {}
);

declare_queue_struct!(
    struct LogDepsQueue<StaticString> {}
);

pub type LogBlock = EventBlock<LogMsgQueue>;

pub type LogStream = EventStream<LogBlock, LogDepsQueue>;
