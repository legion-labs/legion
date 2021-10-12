use transit::prelude::*;

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
    pub level: u8,
    pub msg_len: u32,
    pub msg: *const u8,
}

impl InProcSerialize for LogMsgEvent {}

#[derive(Debug)]
pub struct LogDynMsgEvent {
    pub level: u8,
    pub msg: DynString,
}

impl InProcSerialize for LogDynMsgEvent {
    fn is_size_static() -> bool {
        false
    }

    fn get_value_size(&self) -> Option<u32> {
        Some(self.msg.get_value_size().unwrap() + 1)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        write_pod::<u8>(buffer, &self.level);
        self.msg.write_value(buffer);
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self {
        let level = read_pod::<u8>(ptr);
        let buffer_size = value_size.unwrap();
        let string_ptr = unsafe { ptr.add(1) };
        let msg = <DynString as InProcSerialize>::read_value(string_ptr, Some(buffer_size - 1));
        Self { level, msg }
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
