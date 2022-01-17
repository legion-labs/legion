use crate::{
    read_any, write_any, InProcSerialize, InProcSize, Reflect, StaticString, UserDefinedType,
};

// StringId serializes the value of the pointer and the size
// Also provides a facility to extract a StaticString from it
#[derive(Debug)]
pub struct StringId {
    pub ptr: *const u8,
    pub len: u32,
}

impl std::convert::From<&'static str> for StringId {
    fn from(src: &'static str) -> Self {
        Self {
            len: src.len() as u32,
            ptr: src.as_ptr(),
        }
    }
}

impl std::convert::From<&StringId> for StaticString {
    fn from(src: &StringId) -> Self {
        Self {
            len: src.len,
            ptr: src.ptr,
        }
    }
}

// dummy impl for Reflect
impl Reflect for StringId {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: String::from("StringId"),
            size: 0,
            members: vec![],
        }
    }
}

impl StringId {
    pub fn id(&self) -> u64 {
        self.ptr as u64
    }

    pub const fn rw_size() -> usize {
        std::mem::size_of::<u32>() + std::mem::size_of::<usize>()
    }
}

impl InProcSerialize for StringId {
    const IN_PROC_SIZE: InProcSize = InProcSize::Const(Self::rw_size());

    #[allow(unsafe_code)]
    #[inline(always)]
    fn write_value(&self, buffer: &mut Vec<u8>) {
        write_any(buffer, &self.len);
        write_any(buffer, &self.ptr);
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    unsafe fn read_value(mut ptr: *const u8, _value_size: Option<u32>) -> Self {
        let len = read_any::<u32>(ptr);
        ptr = ptr.add(std::mem::size_of::<u32>() as usize);
        let ptr = read_any::<*const u8>(ptr);
        Self { ptr, len }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_string_id() {
        use super::*;
        let string_id = StringId::from("hello");
        assert_eq!(string_id.len, 5);
        assert_eq!(string_id.ptr, "hello".as_ptr());
        assert_eq!(string_id.id(), "hello".as_ptr() as u64);

        let mut buffer = vec![];
        string_id.write_value(&mut buffer);
        assert_eq!(buffer.len(), StringId::rw_size());

        let string_id = unsafe { StringId::read_value(buffer.as_ptr(), None) };
        assert_eq!(string_id.len, 5);
        assert_eq!(string_id.ptr, "hello".as_ptr());
    }
}
