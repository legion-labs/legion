use std::collections::HashSet;

use lgn_tracing_transit::prelude::*;

use crate::event_block::{EventBlock, ExtractDeps};
use crate::{BeginScopeEvent, EndScopeEvent, EventStream, ReferencedScope, ScopeDesc};

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedScope, StaticString> {}
);

fn record_scope_event_dependencies(
    evt_desc: &'static ScopeDesc,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut ThreadDepsQueue,
) {
    let ptr = evt_desc as *const _ as u64;
    if recorded_deps.insert(ptr) {
        let name = StaticString::from(evt_desc.name);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let filename = StaticString::from(evt_desc.filename);
        if recorded_deps.insert(filename.ptr as u64) {
            deps.push(filename);
        }
        deps.push(ReferencedScope {
            id: ptr,
            name: evt_desc.name.as_ptr(),
            filename: evt_desc.filename.as_ptr(),
            line: evt_desc.line,
        });
    }
}

impl ExtractDeps for ThreadEventQueue {
    type DepsQueue = ThreadDepsQueue;

    fn extract(&self) -> Self::DepsQueue {
        let mut deps = ThreadDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.iter() {
            match x {
                ThreadEventQueueAny::BeginScopeEvent(evt) => {
                    record_scope_event_dependencies(evt.scope, &mut recorded_deps, &mut deps);
                }
                ThreadEventQueueAny::EndScopeEvent(evt) => {
                    record_scope_event_dependencies(evt.scope, &mut recorded_deps, &mut deps);
                }
            }
        }
        deps
    }
}

pub type ThreadBlock = EventBlock<ThreadEventQueue>;
pub type ThreadStream = EventStream<ThreadBlock>;
