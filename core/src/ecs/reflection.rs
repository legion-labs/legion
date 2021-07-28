pub trait Reference {
    fn is_mutable() -> bool;
}

impl<T> Reference for &mut T {
    fn is_mutable() -> bool {
        true
    }
}

impl<T> Reference for &T {
    fn is_mutable() -> bool {
        false
    }
}

pub trait Named {
    fn get_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> Named for &T {
    fn get_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

impl<T> Named for &mut T {
    fn get_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

pub fn get_short_type_name(type_name: &'static str) -> &'static str {
    match type_name.rfind("::") {
        Some(index) => &type_name[(index + 2)..],
        None => type_name,
    }
}
