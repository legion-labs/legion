use crate::{InProcSerialize, Reflect, UserDefinedType};

#[derive(Debug)]
pub struct DynString(pub String);

impl InProcSerialize for DynString {
    fn is_size_static() -> bool {
        false
    }

    fn get_value_size(&self) -> Option<u32> {
        Some(self.0.len() as u32)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(self.0.as_bytes());
    }

    #[allow(unsafe_code)]
    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self {
        let buffer_size = value_size.unwrap();
        let slice = std::ptr::slice_from_raw_parts(ptr, buffer_size as usize);
        unsafe { Self(String::from_utf8((*slice).to_vec()).unwrap()) }
    }
}

impl Reflect for DynString {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: String::from("String"),
            size: 0,
            members: vec![],
        }
    }
}
