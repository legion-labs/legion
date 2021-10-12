use crate::{on_end_scope, GetScopeDesc};

#[derive(Debug)]
pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

pub struct ScopeGuard {
    // the value of the function pointer will identity the scope uniquely within that process instance
    pub get_scope_desc: GetScopeDesc,
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
