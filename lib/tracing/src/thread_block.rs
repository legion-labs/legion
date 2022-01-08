use anyhow::Result;
use lgn_transit::prelude::*;

use crate::event_block::EventBlock;
use crate::{BeginScopeEvent, EndScopeEvent, EventStream, ReferencedScope};

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedScope, StaticString> {}
);

pub type ThreadBlock = EventBlock<ThreadEventQueue>;

pub type ThreadStream = EventStream<ThreadBlock, ThreadDepsQueue>;
