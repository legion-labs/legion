#[allow(unsafe_code)]
pub fn write_pod<T>(buffer: &mut Vec<u8>, value: &T) {
    let ptr = std::ptr::addr_of!(*value).cast::<u8>();
    let slice = std::ptr::slice_from_raw_parts(ptr, std::mem::size_of::<T>());
    unsafe {
        buffer.extend_from_slice(&*slice);
    }
}

#[allow(unsafe_code)]
pub fn read_pod<T>(ptr: *const u8) -> T {
    unsafe { std::ptr::read_unaligned(ptr.cast::<T>()) }
}

pub trait Serialize {
    fn is_size_static() -> bool {
        true
    }

    fn get_value_size(_value: &Self) -> Option<u32> {
        // for POD serialization we don't write the size of each instance
        // the metadata will contain this size
        None
    }

    fn write_value(buffer: &mut Vec<u8>, value: &Self)
    where
        Self: Sized,
    {
        assert!(Self::is_size_static());
        #[allow(clippy::needless_borrow)]
        //clippy complains here but we don't want to move or copy the value
        write_pod::<Self>(buffer, &value);
    }

    fn read_value(ptr: *const u8, _value_size: Option<u32>) -> Self
    where
        Self: Sized,
    {
        read_pod::<Self>(ptr)
    }
}
