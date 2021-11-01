use crate::{
    compress, event_block::EventBlock, BeginScopeEvent, EncodedBlock, EndScopeEvent, EventStream,
    ReferencedScope, ScopeEvent, StreamBlock,
};
use anyhow::Result;
use std::collections::HashSet;
use transit::prelude::*;

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedScope, StaticString> {}
);

pub type ThreadBlock = EventBlock<ThreadEventQueue>;

fn record_scope_event_dependencies<T: ScopeEvent>(
    evt: &T,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut ThreadDepsQueue,
) {
    let get_scope = evt.get_scope();
    let ptr = get_scope as usize as u64;
    if recorded_deps.insert(ptr) {
        let desc = get_scope();
        let name = StaticString::from(desc.name);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let filename = StaticString::from(desc.filename);
        if recorded_deps.insert(filename.ptr as u64) {
            deps.push(filename);
        }
        deps.push(ReferencedScope {
            id: ptr,
            name: desc.name.as_ptr(),
            filename: desc.filename.as_ptr(),
            line: desc.line,
        });
    }
}

impl StreamBlock for ThreadBlock {
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = ThreadDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.events.iter() {
            match x {
                ThreadEventQueueAny::BeginScopeEvent(evt) => {
                    record_scope_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
                ThreadEventQueueAny::EndScopeEvent(evt) => {
                    record_scope_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
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

pub type ThreadStream = EventStream<ThreadBlock, ThreadDepsQueue>;
