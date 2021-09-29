use transit::*;

#[derive(Debug, TransitReflect)]
pub struct LogMsgEvent {
    pub level: i32,
    pub msg: &'static str,
}

impl Serialize for LogMsgEvent {}

#[derive(Debug, TransitReflect)]
pub struct NullEvent {}

impl Serialize for NullEvent {}

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent, NullEvent> {}
);

// StaticString serializes the value of the pointer and the contents of the string
#[derive(Debug)]
pub struct StaticString(pub &'static str);

// dummy impl for Reflect
impl Reflect for StaticString {
    fn reflect() -> UserDefinedType {
        UserDefinedType {
            name: "StaticString",
            size: 0,
            members: vec![],
        }
    }
}

impl Serialize for StaticString {
    fn is_size_static() -> bool {
        false
    }

    fn get_value_size(&self) -> Option<u32> {
        let id_size = std::mem::size_of::<usize>() as u32;
        Some(self.0.len() as u32 + id_size)
    }

    fn write_value(&self, buffer: &mut Vec<u8>) {
        write_pod(buffer, &self.0.as_ptr());
        buffer.extend_from_slice(self.0.as_bytes());
    }

    #[allow(unsafe_code)]
    fn read_value(ptr: *const u8, value_size_opt: Option<u32>) -> Self {
        let id_size = std::mem::size_of::<usize>() as u32;
        let value_size = value_size_opt.unwrap();
        assert!(id_size <= value_size);
        let buffer_size = value_size - id_size;
        let static_buffer_ptr = read_pod::<*const u8>(ptr);
        let slice = std::ptr::slice_from_raw_parts(static_buffer_ptr, buffer_size as usize);
        unsafe { Self(std::str::from_utf8(&*slice).unwrap()) }
    }
}

declare_queue_struct!(
    struct DepQueue<StaticString> {}
);

#[test]
fn test_deps() {
    let mut q = LogMsgQueue::new(1024);
    q.push(LogMsgEvent {
        level: 0,
        msg: "test_msg",
    });
    q.push(NullEvent {});
    q.push(LogMsgEvent {
        level: 0,
        msg: "__test",
    });

    let reflection = LogMsgQueue::reflect_contained();
    dbg!(reflection);

    let mut deps = DepQueue::new(1024);

    for x in q.iter() {
        match x {
            LogMsgQueueAny::LogMsgEvent(evt) => {
                deps.push(StaticString(evt.msg));
            }
            LogMsgQueueAny::NullEvent(_evt) => {}
        }
    }

    assert_eq!(40, deps.len_bytes());

    for x in deps.iter() {
        dbg!(x);
    }
}
