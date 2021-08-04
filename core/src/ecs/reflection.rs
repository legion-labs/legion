use std::any::TypeId;

pub fn shorten_type_name(type_name: &'static str) -> &'static str {
    match type_name.rsplit("::").next() {
        Some(last_segment) => last_segment,
        None => type_name,
    }
}

pub trait Reference: 'static {
    type Inner;

    fn get_type_id() -> TypeId {
        TypeId::of::<Self::Inner>()
    }

    fn get_name() -> &'static str {
        std::any::type_name::<Self::Inner>()
    }

    fn get_short_type_name() -> &'static str {
        shorten_type_name(Self::get_name())
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
