use core::arch::x86_64::_rdtsc;

pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

type GetScopeDesc = fn() -> ScopeDesc;

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
        let scope_desc = (self.get_scope_desc)();
        println!(
            "done {} in {} at line {} at time {}",
            scope_desc.name,
            scope_desc.filename,
            scope_desc.line,
            now()
        );
    }
}

#[macro_export]
macro_rules! trace_scope {
    () => {
        fn _scope() -> ScopeDesc {
            // no need to build the ScopeDesc object until we serialize the events
            fn outer_function_name() -> &'static str {
                let inner = type_name_of(&_scope);
                static TAIL_LEN: usize = "_scope".len() + 2;
                &inner[0..inner.len() - TAIL_LEN]
            }

            ScopeDesc {
                name: outer_function_name(),
                filename: file!(),
                line: line!(),
            }
        }
        let guard = ScopeGuard {
            get_scope_desc: _scope,
        };
        println!("begin @ {}", now());
    };
}

pub struct BeginScopeEvent {
    pub time: u64,
    pub get_scope_desc: GetScopeDesc,
}

pub struct EndScopeEvent {
    pub time: u64,
    pub get_scope_desc: GetScopeDesc,
}
