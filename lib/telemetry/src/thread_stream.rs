use crate::{
    make_queue_metedata, on_end_scope, GetScopeDesc, Stream, StreamInfo, ThreadBlock,
    ThreadDepsQueue, ThreadEventQueue, ThreadEventQueueTypeIndex,
};
use core::arch::x86_64::_rdtsc;
use std::sync::Arc;
use transit::{InProcSerialize, IterableQueue, Member, TransitReflect, UserDefinedType};

#[derive(Debug)]
pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

#[derive(Debug, TransitReflect)]
pub struct ReferencedScope {
    pub id: u64,
    pub name: *const u8,
    pub filename: *const u8,
    pub line: u32,
}

impl InProcSerialize for ReferencedScope {}

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

pub struct ThreadStream {
    current_block: Arc<ThreadBlock>,
    initial_size: usize,
    stream_id: String,
    process_id: String,
}

impl ThreadStream {
    pub fn new(buffer_size: usize, process_id: String) -> Self {
        let stream_id = uuid::Uuid::new_v4().to_string();
        Self {
            current_block: Arc::new(ThreadBlock::new(buffer_size, stream_id.clone())),
            initial_size: buffer_size,
            stream_id,
            process_id,
        }
    }

    pub fn replace_block(&mut self, new_block: Arc<ThreadBlock>) -> Arc<ThreadBlock> {
        let old_block = self.current_block.clone();
        self.current_block = new_block;
        old_block
    }

    pub fn push_event<T>(&mut self, event: T)
    where
        T: InProcSerialize + ThreadEventQueueTypeIndex,
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
