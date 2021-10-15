pub fn round_size_up_to_alignment_u32(size: u32, required_alignment: u32) -> u32 {
    assert!(required_alignment > 0);
    ((size + required_alignment - 1) / required_alignment) * required_alignment
}

pub fn round_size_up_to_alignment_u64(size: u64, required_alignment: u64) -> u64 {
    assert!(required_alignment > 0);
    ((size + required_alignment - 1) / required_alignment) * required_alignment
}

pub fn round_size_up_to_alignment_usize(size: usize, required_alignment: usize) -> usize {
    assert!(required_alignment > 0);
    ((size + required_alignment - 1) / required_alignment) * required_alignment
}

#[allow(unsafe_code)]
pub fn any_as_bytes<T: Copy>(data: &T) -> &[u8] {
    let ptr: *const T = data;
    let ptr = ptr.cast::<u8>();
    let slice: &[u8] = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };

    slice
}

pub fn slice_size_in_bytes<T>(slice: &[T]) -> usize {
    let range = slice.as_ptr_range();
    (range.end.cast::<u8>() as usize) - (range.start.cast::<u8>() as usize)
}

#[allow(unsafe_code)]
pub fn write_any<T>(buffer: &mut Vec<u8>, value: &T) {
    let ptr = std::ptr::addr_of!(*value).cast::<u8>();
    let slice = std::ptr::slice_from_raw_parts(ptr, std::mem::size_of::<T>());
    unsafe {
        buffer.extend_from_slice(&*slice);
    }
}
#[allow(unsafe_code)]
pub fn read_any<T>(ptr: *const u8) -> T {
    unsafe { std::ptr::read_unaligned(ptr.cast::<T>()) }
}
