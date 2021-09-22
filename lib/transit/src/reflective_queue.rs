use crate::UserDefinedType;

pub trait ReflectiveQueue {
    fn reflect_contained() -> Vec<UserDefinedType>;
}
