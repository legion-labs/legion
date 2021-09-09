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

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let scope_desc = (self.get_scope_desc)();
        println!(
            "done {} in {} at line {}",
            scope_desc.name, scope_desc.filename, scope_desc.line
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
    };
}
