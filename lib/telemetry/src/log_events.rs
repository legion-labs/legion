use lgn_transit::prelude::*;
use lgn_utils::memory::{read_any, write_any};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl std::convert::From<log::Level> for LogLevel {
    fn from(src: log::Level) -> Self {
        match src {
            log::Level::Error => Self::Error,
            log::Level::Warn => Self::Warning,
            _ => Self::Info,
        }
    }
}

#[derive(Debug, TransitReflect)]
pub struct LogMsgEvent {
    pub time: i64,
    pub level: u8,
    pub msg_len: u32,
    pub msg: *const u8,
}

impl InProcSerialize for LogMsgEvent {}

#[derive(Debug)]
pub struct LogDynMsgEvent {
    pub time: i64,
    pub level: u8,
    pub msg: DynString,
}

impl InProcSerialize for LogDynMsgEvent {
    fn is_size_static() -> bool {
        false
    }

    fn get_value_size(&self) -> Option<u32> {
        Some(self.msg.get_value_size().unwrap() + 1 + std::mem::size_of::<u64>() as u32)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        write_any(buffer, &self.time);
        write_any(buffer, &self.level);
        self.msg.write_value(buffer);
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self {
        let time = read_any::<i64>(ptr);
        let level_offset = std::mem::size_of::<u64>();
        let level = unsafe { read_any::<u8>(ptr.add(level_offset)) };
        let buffer_size = value_size.unwrap();
        let string_offset = 1 + level_offset;
        let string_ptr = unsafe { ptr.add(string_offset) };
        let msg = <DynString as InProcSerialize>::read_value(
            string_ptr,
            Some(buffer_size - string_offset as u32),
        );
        Self { time, level, msg }
    }
}

//todo: change this interface to make clear that there are two serialization strategies: pod and custom
impl Reflect for LogDynMsgEvent {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: String::from("LogDynMsgEvent"),
            size: 0,
            members: vec![],
        }
    }
}
