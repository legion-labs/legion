#[allow(unsafe_code)]
#[inline]
pub fn write_any<T>(buffer: &mut Vec<u8>, value: &T) {
    let ptr = std::ptr::addr_of!(*value).cast::<u8>();
    let slice = std::ptr::slice_from_raw_parts(ptr, std::mem::size_of::<T>());
    unsafe {
        buffer.extend_from_slice(&*slice);
    }
}

#[allow(unsafe_code)]
/// Helper function to read a u* pointer to a value of type T.
///
/// # Safety
/// ptr must be valid it's size and it's memory size must be the size
/// of T or higher.
#[inline]
pub unsafe fn read_any<T>(ptr: *const u8) -> T {
    std::ptr::read_unaligned(ptr.cast::<T>())
}

// InProcSerialize is used by the heterogeneous queue to write objects in its
// buffer serialized objects can have references with static lifetimes
pub trait InProcSerialize {
    const IS_CONST_SIZE: bool = true;

    fn get_value_size(&self) -> Option<u32> {
        // for POD serialization we don't write the size of each instance
        // the metadata will contain this size
        None
    }

    fn write_value(&self, buffer: &mut Vec<u8>)
    where
        Self: Sized,
    {
        assert!(Self::IS_CONST_SIZE);
        #[allow(clippy::needless_borrow)]
        //clippy complains here but we don't want to move or copy the value
        write_any::<Self>(buffer, &self);
    }

    // read_value allows to read objects from the same process they were stored in
    // i.e. iterating in the heterogenous queue
    /// # Safety
    /// This is called from the serializer context that that uses `value_size`
    /// call to make sure that the proper size is used
    #[allow(unsafe_code)]
    unsafe fn read_value(ptr: *const u8, _value_size: Option<u32>) -> Self
    where
        Self: Sized,
    {
        read_any::<Self>(ptr)
    }
}
