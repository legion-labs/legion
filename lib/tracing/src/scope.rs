use std::marker::PhantomData;

use crate::on_end_scope;

#[derive(Debug)]
pub struct ScopeDesc {
    pub name: &'static str,
    pub filename: &'static str,
    pub line: u32,
}

pub struct ScopeGuard {
    // the value of the function pointer will identity the scope uniquely within that process
    // instance
    pub scope_desc: &'static ScopeDesc,
    pub _dummy_ptr: PhantomData<*mut u8>, // to mark the object as !Send
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        on_end_scope(self.scope_desc);
    }
}

//pub const fn type_name_of<T>(_: &T) -> &'static str {
//    //until type_name_of_val is out of nightly-only
//    std::any::type_name::<T>()
//}

/// Returns the name of the calling function without a long module path prefix.
#[macro_export]
macro_rules! function_name {
    () => {{
        // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // `3` is the length of the `::f`.
        &name[..name.len() - 3]
    }};
}

#[macro_export]
macro_rules! trace_scope {
    ($scope_id:ident, $name:expr) => {
        static $scope_id: $crate::ScopeDesc = $crate::ScopeDesc {
            name: $name,
            filename: file!(),
            line: line!(),
        };
        let guard_named = $crate::ScopeGuard {
            scope_desc: &$scope_id,
            _dummy_ptr: std::marker::PhantomData::default(),
        };
        $crate::on_begin_scope(&$scope_id);
    };
    ($name:expr) => {
        $crate::trace_scope!(_SCOPE_NAMED, $name);
    };
}
