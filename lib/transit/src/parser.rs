use std::collections::HashMap;

use crate::*;
use anyhow::*;

#[derive(Debug, Clone)]
pub struct Object {
    pub type_name: String,
    pub members: Vec<(String, Value)>,
}

impl Object {
    pub fn get<T>(&self, member_name: &str) -> Result<T>
    where
        T: TansitValue,
    {
        for m in &self.members {
            if m.0 == member_name {
                return T::get(&m.1);
            }
        }
        bail!("member {} not found", member_name);
    }
}

pub trait TansitValue {
    fn get(value: &Value) -> Result<Self>
    where
        Self: Sized;
}

impl TansitValue for u8 {
    fn get(value: &Value) -> Result<Self> {
        if let Value::U8(val) = value {
            Ok(*val)
        } else {
            bail!("bad type cast u8 for value {:?}", value);
        }
    }
}

impl TansitValue for String {
    fn get(value: &Value) -> Result<Self> {
        if let Value::String(val) = value {
            Ok(val.clone())
        } else {
            bail!("bad type cast String for value {:?}", value);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Object(Object),
    U8(u8),
    U32(u32),
    None,
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

fn parse_custom_instance(
    udt: &UserDefinedType,
    _dependencies: &HashMap<usize, Value>,
    offset: usize,
    object_size: usize,
    buffer: &[u8],
) -> Vec<(String, Value)> {
    match udt.name.as_str() {
        "LogDynMsgEvent" => unsafe {
            let level_ptr = buffer.as_ptr().add(offset);
            let level = read_pod::<u8>(level_ptr);
            let msg = <DynString as Serialize>::read_value(
                buffer.as_ptr().add(offset + 1),
                Some((object_size - 1) as u32),
            );
            vec![
                (String::from("level"), Value::U8(level)),
                (String::from("msg"), Value::String(msg.0)),
            ]
        },
        other => {
            println!("unknown custom object {}", other);
            Vec::new()
        }
    }
}

fn parse_pod_instance(
    udt: &UserDefinedType,
    dependencies: &HashMap<usize, Value>,
    offset: usize,
    buffer: &[u8],
) -> Vec<(String, Value)> {
    udt.members
        .iter()
        .map(|member_meta| {
            let name = member_meta.name.clone();
            let type_name = member_meta.type_name.clone();
            let value = if member_meta.is_reference {
                assert_eq!(std::mem::size_of::<usize>(), member_meta.size);
                let key =
                    read_pod::<usize>(unsafe { buffer.as_ptr().add(offset + member_meta.offset) });
                if let Some(v) = dependencies.get(&key) {
                    v.clone()
                } else {
                    println!("dependency not found: {}", key);
                    Value::None
                }
            } else {
                match type_name.as_str() {
                    "u8" => {
                        assert_eq!(std::mem::size_of::<u8>(), member_meta.size);
                        Value::U8(read_pod::<u8>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    "u32" => {
                        assert_eq!(std::mem::size_of::<u32>(), member_meta.size);
                        Value::U32(read_pod::<u32>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    unknown_member_type => {
                        println!("unknown member type {}", unknown_member_type);
                        Value::None
                    }
                }
            };
            (name, value)
        })
        .collect()
}

pub fn parse_objects<F>(
    dependencies: &HashMap<usize, Value>,
    udts: &[UserDefinedType],
    buffer: &[u8],
    mut fun: F,
) -> Result<()>
where
    F: FnMut(Value),
{
    let mut offset = 0;
    while offset < buffer.len() {
        let type_index = buffer[offset] as usize;
        if type_index >= udts.len() {
            bail!("Invalid type index parsing transit objects: {}", type_index);
        }
        offset += 1;
        let udt = &udts[type_index];
        let (object_size, is_size_dynamic) = match udt.size {
            0 => {
                //dynamic size
                unsafe {
                    let size_ptr = buffer.as_ptr().add(offset);
                    let obj_size = read_pod::<u32>(size_ptr);
                    offset += std::mem::size_of::<u32>();
                    (obj_size as usize, true)
                }
            }
            static_size => (static_size, false),
        };
        let instance_members = if is_size_dynamic {
            parse_custom_instance(udt, dependencies, offset, object_size, buffer)
        } else {
            parse_pod_instance(udt, dependencies, offset, buffer)
        };
        let instance = Object {
            type_name: udt.name.clone(),
            members: instance_members,
        };
        fun(Value::Object(instance));
        offset += object_size;
    }
    Ok(())
}
