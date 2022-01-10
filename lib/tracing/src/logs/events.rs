use lgn_tracing_transit::prelude::*;
use lgn_utils::memory::{read_any, write_any};

use crate::Level;

#[derive(Debug)]
pub struct LogMetadata {
    pub level: u32,
    pub fmt_str: &'static str,
    pub target: &'static str,
    pub module_path: &'static str,
    pub file: &'static str,
    pub line: u32,
}

impl LogMetadata {
    pub fn level(&self) -> Level {
        Level::from_u32(self.level).unwrap()
    }
}

#[derive(Debug, TransitReflect)]
pub struct LogStaticStrEvent {
    pub desc: &'static LogMetadata,
    pub time: i64,
}

impl InProcSerialize for LogStaticStrEvent {}

#[derive(Debug)]
pub struct LogStringEvent {
    pub desc: &'static LogMetadata,
    pub time: i64,
    pub dyn_str: DynString,
}

impl InProcSerialize for LogStringEvent {
    const IS_CONST_SIZE: bool = false;

    fn get_value_size(&self) -> Option<u32> {
        Some(
            std::mem::size_of::<usize>() as u32
                + self.dyn_str.get_value_size().unwrap()
                + std::mem::size_of::<u64>() as u32,
        )
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        let desc_id = self.desc as *const _ as usize;
        write_any(buffer, &desc_id);
        write_any(buffer, &self.time);
        self.dyn_str.write_value(buffer);
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self {
        let desc_id = read_any::<usize>(ptr);
        let desc = unsafe { &*(desc_id as *const LogMetadata) };
        let time_offset = std::mem::size_of::<usize>();
        let time = unsafe { read_any::<i64>(ptr.add(time_offset)) };
        let buffer_size = value_size.unwrap();
        let string_offset = std::mem::size_of::<i64>() + time_offset;
        let string_ptr = unsafe { ptr.add(string_offset) };
        let msg = <DynString as InProcSerialize>::read_value(
            string_ptr,
            Some(buffer_size - string_offset as u32),
        );
        Self {
            desc,
            time,
            dyn_str: msg,
        }
    }
}

//todo: change this interface to make clear that there are two serialization
// strategies: pod and custom
impl Reflect for LogStringEvent {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: String::from("LogStringEvent"),
            size: 0,
            members: vec![],
        }
    }
}

#[derive(Debug, TransitReflect)]
pub struct LogStaticStrInteropEvent {
    pub time: i64,
    pub level: u32,
    pub target_len: u32,
    pub target: *const u8,
    pub msg_len: u32,
    pub msg: *const u8,
}

impl InProcSerialize for LogStaticStrInteropEvent {}

#[derive(Debug)]
pub struct LogStringInteropEvent {
    pub time: i64,
    pub level: u32,
    pub target_len: u32,
    pub target: *const u8,
    pub msg: DynString,
}

impl InProcSerialize for LogStringInteropEvent {
    const IS_CONST_SIZE: bool = false;

    fn get_value_size(&self) -> Option<u32> {
        Some(
            self.msg.get_value_size().unwrap()
                + 2 * (std::mem::size_of::<u32>() + std::mem::size_of::<u64>()) as u32,
        )
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        write_any(buffer, &self.time);
        write_any(buffer, &self.level);
        write_any(buffer, &self.target_len);
        write_any(buffer, &self.target);
        self.msg.write_value(buffer);
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self {
        let time = read_any::<i64>(ptr);
        let level_offset = std::mem::size_of::<i64>();
        let level = unsafe { read_any::<u32>(ptr.add(level_offset)) };
        let target_len_offset = level_offset + std::mem::size_of::<u32>();
        let target_len = unsafe { read_any::<u32>(ptr.add(target_len_offset)) };
        let target_offset = target_len_offset + std::mem::size_of::<u32>();
        let target = unsafe { read_any::<*const u8>(ptr.add(target_offset)) };
        let buffer_size = value_size.unwrap();
        let string_offset = std::mem::size_of::<*const u8>() + target_offset;
        let string_ptr = unsafe { ptr.add(string_offset) };
        let msg = <DynString as InProcSerialize>::read_value(
            string_ptr,
            Some(buffer_size - string_offset as u32),
        );
        Self {
            time,
            level,
            target_len,
            target,
            msg,
        }
    }
}

//todo: change this interface to make clear that there are two serialization
// strategies: pod and custom
impl Reflect for LogStringInteropEvent {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: String::from("LogStringInteropEvent"),
            size: 0,
            members: vec![],
        }
    }
}

#[derive(Debug, TransitReflect)]
pub struct LogMetadataRecord {
    pub id: u64,
    pub fmt_str: *const u8,
    pub target: *const u8,
    pub module_path: *const u8,
    pub file: *const u8,
    pub line: u32,
    pub level: u32,
}

impl InProcSerialize for LogMetadataRecord {}
