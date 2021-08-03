use std::any::TypeId;

pub trait Reference: 'static {
    type Inner;

    fn get_type_id() -> TypeId {
        TypeId::of::<Self::Inner>()
    }

    fn get_name() -> &'static str {
        std::any::type_name::<Self::Inner>()
    }

    fn get_short_type_name() -> &'static str {
        let name = Self::get_name();
        match name.rsplit("::").next() {
            Some(name) => name,
            None => name,
        }
    }

    fn is_mutable() -> bool;
}

impl<T> Reference for &'static T {
    type Inner = T;

    fn is_mutable() -> bool {
        false
    }
}

impl<T> Reference for &'static mut T {
    type Inner = T;

    fn is_mutable() -> bool {
        true
    }
}
