use crate::*;
use core::arch::x86_64::_rdtsc;
use std::sync::Arc;
use transit::*;

pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

pub type GetScopeDesc = fn() -> ScopeDesc;

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
        // let scope_desc = (self.get_scope_desc)();
        // println!(
        //     "done {} in {} at line {} at time {}",
        //     scope_desc.name,
        //     scope_desc.filename,
        //     scope_desc.line,
        //     now()
        // );
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

#[derive(Debug)]
pub struct BeginScopeEvent {
    pub time: u64,
    pub get_scope_desc: GetScopeDesc,
}

impl Serialize for BeginScopeEvent {}

#[derive(Debug)]
pub struct EndScopeEvent {
    pub time: u64,
    pub get_scope_desc: GetScopeDesc,
}

impl Serialize for EndScopeEvent {}

declare_queue_struct!(
    struct ThreadEventQueue<BeginScopeEvent, EndScopeEvent> {}
);

#[derive(Debug)]
pub struct ThreadEventBlock {
    pub events: ThreadEventQueue,
}

impl ThreadEventBlock {
    pub fn new(buffer_size: usize) -> Self {
        let events = ThreadEventQueue::new(buffer_size);
        Self { events }
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
            current_block: Arc::new(ThreadEventBlock::new(buffer_size)),
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
        StreamInfo {
            process_id: self.process_id.clone(),
            stream_id: self.stream_id.clone(),
            dependencies_metadata: Some(telemetry_ingestion_proto::ContainerMetadata {
                types: vec![],
            }),
            objects_metadata: Some(telemetry_ingestion_proto::ContainerMetadata { types: vec![] }),
            tags: vec![String::from("cpu")],
        }
    }
}
