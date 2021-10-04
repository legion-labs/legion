use std::collections::HashMap;

use crate::*;
use anyhow::*;

#[derive(Debug)]
pub enum Value {
    String(String),
}

pub fn parse_dependencies<F>(udts: &[UserDefinedType], buffer: &[u8], mut fun: F) -> Result<()>
where
    F: FnMut(usize, Value),
{
    let mut offset = 0;
    while offset < buffer.len() {
        let type_index = buffer[offset] as usize;
        if type_index >= udts.len() {
            bail!(
                "Invalid type index parsing transit dependencies: {}",
                type_index
            );
        }
        offset += 1;
        let udt = &udts[type_index];
        let object_size = match udt.size {
            0 => {
                //dynamic size
                unsafe {
                    let size_ptr = buffer.as_ptr().add(offset);
                    let obj_size = read_pod::<u32>(size_ptr);
                    offset += std::mem::size_of::<u32>();
                    obj_size as usize
                }
            }
            static_size => static_size,
        };
        match udt.name.as_str() {
            "StaticString" => unsafe {
                let id_ptr = buffer.as_ptr().add(offset);
                let string_id = read_pod::<usize>(id_ptr);
                let nb_utf8_bytes = object_size - std::mem::size_of::<usize>();
                let utf8_ptr = buffer.as_ptr().add(offset + std::mem::size_of::<usize>());
                let slice = std::ptr::slice_from_raw_parts(utf8_ptr, nb_utf8_bytes);
                let string = String::from(std::str::from_utf8(&*slice).unwrap());
                fun(string_id, Value::String(string));
            },
            unknown_type => {
                println!("unknown type {}", unknown_type);
            }
        }
        offset += object_size;
    }
    Ok(())
}

pub fn read_dependencies(udts: &[UserDefinedType], buffer: &[u8]) -> Result<HashMap<usize, Value>> {
    let mut hash = HashMap::new();
    parse_dependencies(udts, buffer, |id, value| {
        hash.insert(id, value);
    })?;
    Ok(hash)
}
