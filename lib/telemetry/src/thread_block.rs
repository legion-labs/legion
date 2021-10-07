use crate::*;
use anyhow::*;
use std::collections::HashSet;
use transit::*;

pub type GetScopeDesc = fn() -> ScopeDesc;

trait ScopeEvent {
    fn get_time(&self) -> u64;
    fn get_scope(&self) -> GetScopeDesc;
}

#[derive(Debug, TransitReflect)]
pub struct BeginScopeEvent {
    pub time: u64,
    pub scope: fn() -> ScopeDesc,
}

impl InProcSerialize for BeginScopeEvent {}
impl ScopeEvent for BeginScopeEvent {
    fn get_time(&self) -> u64 {
        self.time
    }

    fn get_scope(&self) -> GetScopeDesc {
        self.scope
    }
}

#[derive(Debug, TransitReflect)]
pub struct EndScopeEvent {
    pub time: u64,
    pub scope: fn() -> ScopeDesc,
}

impl InProcSerialize for EndScopeEvent {}

impl ScopeEvent for EndScopeEvent {
    fn get_time(&self) -> u64 {
        self.time
    }

    fn get_scope(&self) -> GetScopeDesc {
        self.scope
    }
}

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedScope, StaticString> {}
);

#[derive(Debug)]
pub struct ThreadBlock {
    pub stream_id: String,
    pub begin: DualTime,
    pub events: ThreadEventQueue,
    pub end: Option<DualTime>,
}

impl ThreadBlock {
    pub fn new(buffer_size: usize, stream_id: String) -> Self {
        let events = ThreadEventQueue::new(buffer_size);
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

        let payload = telemetry_ingestion_proto::BlockPayload {
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
