use std::{collections::HashMap, hash::BuildHasher};

use anyhow::{bail, Result};
use lgn_utils::memory::read_any;

use crate::{DynString, InProcSerialize, UserDefinedType};

#[derive(Debug, Clone)]
pub struct Object {
    pub type_name: String,
    pub members: Vec<(String, Value)>,
}

impl Object {
    pub fn get<T>(&self, member_name: &str) -> Result<T>
    where
        T: TransitValue,
    {
        for m in &self.members {
            if m.0 == member_name {
                return T::get(&m.1);
            }
        }
        bail!("member {} not found", member_name);
    }

    pub fn get_ref(&self, member_name: &str) -> Result<&Value> {
        for m in &self.members {
            if m.0 == member_name {
                return Ok(&m.1);
            }
        }
        bail!("member {} not found", member_name);
    }
}

pub trait TransitValue {
    fn get(value: &Value) -> Result<Self>
    where
        Self: Sized;
}

impl TransitValue for u8 {
    fn get(value: &Value) -> Result<Self> {
        if let Value::U8(val) = value {
            Ok(*val)
        } else {
            bail!("bad type cast u8 for value {:?}", value);
        }
    }
}

impl TransitValue for u32 {
    fn get(value: &Value) -> Result<Self> {
        if let Value::U32(val) = value {
            Ok(*val)
        } else {
            bail!("bad type cast u32 for value {:?}", value);
        }
    }
}

impl TransitValue for u64 {
    fn get(value: &Value) -> Result<Self> {
        match value {
            Value::I64(val) => Ok(*val as Self),
            Value::U64(val) => Ok(*val),
            _ => {
                bail!("bad type cast u64 for value {:?}", value)
            }
        }
    }
}

impl TransitValue for i64 {
    #[allow(clippy::cast_possible_wrap)]
    fn get(value: &Value) -> Result<Self> {
        match value {
            Value::I64(val) => Ok(*val),
            Value::U64(val) => Ok(*val as Self),
            _ => {
                bail!("bad type cast i64 for value {:?}", value)
            }
        }
    }
}

impl TransitValue for f64 {
    fn get(value: &Value) -> Result<Self> {
        if let Value::F64(val) = value {
            Ok(*val)
        } else {
            bail!("bad type cast f64 for value {:?}", value);
        }
    }
}

impl TransitValue for String {
    fn get(value: &Value) -> Result<Self> {
        if let Value::String(val) = value {
            Ok(val.clone())
        } else {
            bail!("bad type cast String for value {:?}", value);
        }
    }
}

impl TransitValue for Object {
    fn get(value: &Value) -> Result<Self> {
        if let Value::Object(val) = value {
            Ok(val.clone())
        } else {
            bail!("bad type cast String for value {:?}", value);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String), //todo: change to ref-counted
    Object(Object), //todo: change to ref-counted
    U8(u8),
    U32(u32),
    U64(u64),
    I64(i64),
    F64(f64),
    None,
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = &self {
            Some(s.as_str())
        } else {
            None
        }
    }
}

pub fn read_dependencies(udts: &[UserDefinedType], buffer: &[u8]) -> Result<HashMap<u64, Value>> {
    let mut hash = HashMap::new();
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
                    let obj_size = read_any::<u32>(size_ptr);
                    offset += std::mem::size_of::<u32>();
                    obj_size as usize
                }
            }
            static_size => static_size,
        };
        if udt.name == "StaticString" {
            unsafe {
                let id_ptr = buffer.as_ptr().add(offset);
                let string_id = read_any::<u64>(id_ptr);
                let nb_utf8_bytes = object_size - std::mem::size_of::<usize>();
                let utf8_ptr = buffer.as_ptr().add(offset + std::mem::size_of::<usize>());
                let slice = std::ptr::slice_from_raw_parts(utf8_ptr, nb_utf8_bytes);
                let string = String::from(std::str::from_utf8(&*slice).unwrap());
                let insert_res = hash.insert(string_id, Value::String(string));
                assert!(insert_res.is_none());
            }
        } else {
            assert!(udt.size > 0);
            let instance = parse_pod_instance(udt, &hash, offset, buffer);
            let insert_res = hash.insert(instance.get::<u64>("id")?, Value::Object(instance));
            assert!(insert_res.is_none());
        }
        offset += object_size;
    }

    Ok(hash)
}

fn parse_custom_instance<S>(
    udt: &UserDefinedType,
    _dependencies: &HashMap<u64, Value, S>,
    offset: usize,
    object_size: usize,
    buffer: &[u8],
) -> Object
where
    S: BuildHasher,
{
    let members = match udt.name.as_str() {
        // todo: move out of transit lib.
        // LogDynMsgEvent belongs to the legion-telemetry lib
        // we need a way to inject the serialization logic of custom objects
        "LogDynMsgEvent" => unsafe {
            let time_ptr = buffer.as_ptr().add(offset);
            let time = read_any::<u64>(time_ptr);
            let level_ptr = buffer.as_ptr().add(offset + 8);
            let level = read_any::<u8>(level_ptr);
            let msg_offset = 8 + 1;
            let msg = <DynString as InProcSerialize>::read_value(
                buffer.as_ptr().add(offset + msg_offset),
                Some((object_size - msg_offset) as u32),
            );
            vec![
                (String::from("time"), Value::U64(time)),
                (String::from("level"), Value::U8(level)),
                (String::from("msg"), Value::String(msg.0)),
            ]
        },
        other => {
            println!("unknown custom object {}", other);
            Vec::new()
        }
    };
    Object {
        type_name: udt.name.clone(),
        members,
    }
}

fn parse_pod_instance<S>(
    udt: &UserDefinedType,
    dependencies: &HashMap<u64, Value, S>,
    offset: usize,
    buffer: &[u8],
) -> Object
where
    S: BuildHasher,
{
    let members = udt
        .members
        .iter()
        .map(|member_meta| {
            let name = member_meta.name.clone();
            let type_name = member_meta.type_name.clone();
            let value = if member_meta.is_reference {
                assert_eq!(std::mem::size_of::<u64>(), member_meta.size);
                let key =
                    read_any::<u64>(unsafe { buffer.as_ptr().add(offset + member_meta.offset) });
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
                        Value::U8(read_any::<u8>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    "u32" => {
                        assert_eq!(std::mem::size_of::<u32>(), member_meta.size);
                        Value::U32(read_any::<u32>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    "u64" => {
                        assert_eq!(std::mem::size_of::<u64>(), member_meta.size);
                        Value::U64(read_any::<u64>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    "i64" => {
                        assert_eq!(std::mem::size_of::<i64>(), member_meta.size);
                        Value::I64(read_any::<i64>(unsafe {
                            buffer.as_ptr().add(offset + member_meta.offset)
                        }))
                    }
                    "f64" => {
                        assert_eq!(std::mem::size_of::<f64>(), member_meta.size);
                        Value::F64(read_any::<f64>(unsafe {
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
        .collect();
    Object {
        type_name: udt.name.clone(),
        members,
    }
}

// parse_object_buffer calls fun for each object in the buffer until fun returns
// `false`
pub fn parse_object_buffer<F, S>(
    dependencies: &HashMap<u64, Value, S>,
    udts: &[UserDefinedType],
    buffer: &[u8],
    mut fun: F,
) -> Result<()>
where
    F: FnMut(Value) -> bool,
    S: BuildHasher,
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
                    let obj_size = read_any::<u32>(size_ptr);
                    offset += std::mem::size_of::<u32>();
                    (obj_size as usize, true)
                }
            }
            static_size => (static_size, false),
        };
        let instance = if is_size_dynamic {
            parse_custom_instance(udt, dependencies, offset, object_size, buffer)
        } else {
            parse_pod_instance(udt, dependencies, offset, buffer)
        };
        if !fun(Value::Object(instance)) {
            return Ok(());
        }
        offset += object_size;
    }
    Ok(())
}
