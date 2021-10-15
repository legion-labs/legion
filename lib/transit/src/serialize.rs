use legion_utils::memory::{read_any, write_any};

// InProcSerialize is used by the heterogeneous queue to write objects in its buffer
// serialized objects can have references with static lifetimes
pub trait InProcSerialize {
    fn is_size_static() -> bool {
        true
    }

    fn get_value_size(&self) -> Option<u32> {
        // for POD serialization we don't write the size of each instance
        // the metadata will contain this size
        None
    }

    fn write_value(&self, buffer: &mut Vec<u8>)
    where
        Self: Sized,
    {
        assert!(Self::is_size_static());
        #[allow(clippy::needless_borrow)]
        //clippy complains here but we don't want to move or copy the value
        write_any::<Self>(buffer, &self);
    }

    // read_value allows to read objects from the same process they were stored in
    // i.e. iterating in the heterogenous queue
    fn read_value(ptr: *const u8, _value_size: Option<u32>) -> Self
    where
        Self: Sized,
    {
        read_any::<Self>(ptr)
    }
}
