use crate::{on_end_scope, EventStream, GetScopeDesc, ThreadBlock, ThreadDepsQueue};
use core::arch::x86_64::_rdtsc;
use transit::prelude::*;

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

pub type ThreadStream = EventStream<ThreadBlock, ThreadDepsQueue>;
