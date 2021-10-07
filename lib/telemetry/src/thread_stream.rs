use crate::*;
use anyhow::*;
use core::arch::x86_64::_rdtsc;
use std::sync::Arc;
use transit::*;

#[derive(Debug, TransitReflect)]
pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

pub type GetScopeDesc = fn() -> ScopeDesc;

#[derive(Debug, TransitReflect)]
pub struct ReferencedScope {
    pub id: usize,
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

impl Serialize for ReferencedScope {}

pub struct ScopeGuard {
    // the value of the function pointer will identity the scope uniquely within that process instance
    pub get_scope_desc: GetScopeDesc,
}

pub fn now() -> u64 {
    //_rdtsc does not wait for previous instructions to be retired
    // we could use __rdtscp if we needed more precision at the cost of slightly higher overhead
    unsafe { _rdtsc() }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        on_end_scope(self.get_scope_desc);
    }
}

pub fn type_name_of<T>(_: &T) -> &'static str {
    //until type_name_of_val is out of nightly-only
    std::any::type_name::<T>()
}

#[macro_export]
macro_rules! trace_scope {
    () => {
        fn _scope() -> $crate::ScopeDesc {
            // no need to build the ScopeDesc object until we serialize the events
            fn outer_function_name() -> &'static str {
                let inner = $crate::type_name_of(&_scope);
                static TAIL_LEN: usize = "_scope".len() + 2;
                &inner[0..inner.len() - TAIL_LEN]
            }

            $crate::ScopeDesc {
                name: outer_function_name(),
                filename: file!(),
                line: line!(),
            }
        }
        let guard = $crate::ScopeGuard {
            get_scope_desc: _scope,
        };
        $crate::on_begin_scope(_scope);
    };
}

#[derive(Debug, TransitReflect)]
pub struct BeginScopeEvent {
    pub time: u64,
    pub get_scope_desc: fn() -> ScopeDesc,
}

impl Serialize for BeginScopeEvent {}

#[derive(Debug, TransitReflect)]
pub struct EndScopeEvent {
    pub time: u64,
    pub get_scope_desc: fn() -> ScopeDesc,
}

impl Serialize for EndScopeEvent {}

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

declare_queue_struct!(
    struct ThreadDepsQueue<ReferencedScope, StaticString> {}
);

#[derive(Debug)]
pub struct ThreadEventBlock {
    pub stream_id: String,
    pub begin: DualTime,
    pub events: ThreadEventQueue,
    pub end: Option<DualTime>,
}

impl ThreadEventBlock {
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

impl StreamBlock for ThreadEventBlock {
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = ThreadDepsQueue::new(1024 * 1024);
        for x in self.events.iter() {
            match x {
                ThreadEventQueueAny::BeginScopeEvent(evt) => {
                    let ptr = evt.get_scope_desc as usize;
                    let desc = (evt.get_scope_desc)();
                    deps.push(ReferencedScope {
                        id: ptr,
                        name: desc.name,
                        filename: desc.filename,
                        line: desc.line,
                    });
                }
                ThreadEventQueueAny::EndScopeEvent(evt) => {
                    let ptr = evt.get_scope_desc as usize;
                    let desc = (evt.get_scope_desc)();
                    deps.push(ReferencedScope {
                        id: ptr,
                        name: desc.name,
                        filename: desc.filename,
                        line: desc.line,
                    });
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

pub struct ThreadStream {
    current_block: Arc<ThreadEventBlock>,
    initial_size: usize,
    stream_id: String,
    process_id: String,
}

impl ThreadStream {
    pub fn new(buffer_size: usize, process_id: String) -> Self {
        let stream_id = uuid::Uuid::new_v4().to_string();
        Self {
            current_block: Arc::new(ThreadEventBlock::new(buffer_size, stream_id.clone())),
            initial_size: buffer_size,
            stream_id,
            process_id,
        }
    }

    pub fn replace_block(&mut self, new_block: Arc<ThreadEventBlock>) -> Arc<ThreadEventBlock> {
        let old_block = self.current_block.clone();
        self.current_block = new_block;
        old_block
    }

    pub fn push_event<T>(&mut self, event: T)
    where
        T: Serialize + ThreadEventQueueTypeIndex,
    {
        self.get_events_mut().push(event);
    }

    pub fn is_full(&self) -> bool {
        let max_object_size = 1;
        self.current_block.events.len_bytes() + max_object_size > self.initial_size
    }

    fn get_events_mut(&mut self) -> &mut ThreadEventQueue {
        //get_mut_unchecked should be faster
        &mut Arc::get_mut(&mut self.current_block).unwrap().events
    }
}

impl Stream for ThreadStream {
    fn get_stream_info(&self) -> StreamInfo {
        let dependencies_meta = make_queue_metedata::<ThreadDepsQueue>();
        let obj_meta = make_queue_metedata::<ThreadEventQueue>();
        StreamInfo {
            process_id: self.process_id.clone(),
            stream_id: self.stream_id.clone(),
            dependencies_metadata: Some(dependencies_meta),
            objects_metadata: Some(obj_meta),
            tags: vec![String::from("cpu")],
        }
    }

    fn get_stream_id(&self) -> String {
        self.stream_id.clone()
    }
}
