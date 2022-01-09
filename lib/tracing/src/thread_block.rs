use std::collections::HashSet;

use lgn_tracing_transit::prelude::*;

use crate::{
    event_block::{EventBlock, ExtractDeps},
    event_stream::EventStream,
    thread_events::{
        BeginThreadSpanEvent, EndThreadSpanEvent, ReferencedThreadSpanDesc, ThreadSpanDesc,
    },
};

declare_queue_struct!(
    struct ThreadEventQueue<BeginThreadSpanEvent, EndThreadSpanEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedThreadSpanDesc, StaticString> {}
);

fn record_scope_event_dependencies(
    thread_span_desc: &'static ThreadSpanDesc,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut ThreadDepsQueue,
) {
    let thread_span_ptr = thread_span_desc as *const _ as u64;
    if recorded_deps.insert(thread_span_ptr) {
        let name = StaticString::from(thread_span_desc.name);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let target = StaticString::from(thread_span_desc.target);
        if recorded_deps.insert(target.ptr as u64) {
            deps.push(target);
        }
        let module_path = StaticString::from(thread_span_desc.module_path);
        if recorded_deps.insert(module_path.ptr as u64) {
            deps.push(module_path);
        }
        let file = StaticString::from(thread_span_desc.file);
        if recorded_deps.insert(file.ptr as u64) {
            deps.push(file);
        }
        deps.push(ReferencedThreadSpanDesc {
            id: thread_span_ptr,
            name: thread_span_desc.name.as_ptr(),
            target: thread_span_desc.target.as_ptr(),
            module_path: thread_span_desc.module_path.as_ptr(),
            file: thread_span_desc.file.as_ptr(),
            line: thread_span_desc.line,
            lod: thread_span_desc.lod,
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
                ThreadEventQueueAny::BeginThreadSpanEvent(evt) => {
                    record_scope_event_dependencies(
                        evt.thread_span_desc,
                        &mut recorded_deps,
                        &mut deps,
                    );
                }
                ThreadEventQueueAny::EndThreadSpanEvent(evt) => {
                    record_scope_event_dependencies(
                        evt.thread_span_desc,
                        &mut recorded_deps,
                        &mut deps,
                    );
                }
            }
        }
        deps
    }
}

pub type ThreadBlock = EventBlock<ThreadEventQueue>;
pub type ThreadStream = EventStream<ThreadBlock>;
