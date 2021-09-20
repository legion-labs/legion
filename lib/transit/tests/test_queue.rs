use transit::*;

#[derive(TransitReflect, Debug)]
struct MyTestEvent {
    some_64: u64,
    some_32: u32,
}

impl Serialize for MyTestEvent {
    type Value = MyTestEvent;
}

#[derive(TransitReflect, Debug)]
struct OtherEvent {
    some_64: u64,
}

impl Serialize for OtherEvent {
    type Value = OtherEvent;
}

struct DynString {
    pub string: String,
}

impl Serialize for DynString {
    type Value = DynString;

    fn is_size_static() -> bool {
        false
    }

    fn get_value_size(value: &Self::Value) -> Option<u32> {
        Some(value.string.len() as u32)
    }

    #[allow(unsafe_code)]
    fn write_value(buffer: &mut Vec<u8>, value: &DynString) {
        buffer.extend_from_slice(value.string.as_bytes());
    }

    fn read_value(ptr: *const u8, value_size: Option<u32>) -> Self::Value {
        let buffer_size = value_size.unwrap();
        let slice = std::ptr::slice_from_raw_parts(ptr, buffer_size as usize);
        unsafe {
            let string = String::from_utf8((*slice).to_vec()).unwrap();
            DynString { string }
        }
    }
}

declare_queue_struct!(
    struct MyQueue<MyTestEvent, OtherEvent, DynString> {}
);

#[test]
fn test_queue() {
    assert!(<MyTestEvent as Serialize>::is_size_static());
    assert!(<OtherEvent as Serialize>::is_size_static());
    assert!(!<DynString as Serialize>::is_size_static());

    let mut q = MyQueue::new(1024);
    q.push_my_test_event(MyTestEvent {
        some_64: 2,
        some_32: 3,
    });
    assert_eq!(17, q.len_bytes());

    q.push_other_event(OtherEvent { some_64: 3 });
    assert_eq!(26, q.len_bytes());

    q.push_dyn_string(DynString {
        string: String::from("allo"),
    });
    assert_eq!(35, q.len_bytes());

    if let MyQueueAny::MyTestEvent(e) = q.read_value_at_offset(0) {
        assert_eq!(e.some_64, 2);
        assert_eq!(e.some_32, 3);
    } else {
        panic!("wrong enum type");
    }

    if let MyQueueAny::OtherEvent(e) = q.read_value_at_offset(17) {
        assert_eq!(e.some_64, 3);
    } else {
        panic!("wrong enum type");
    }

    if let MyQueueAny::DynString(s) = q.read_value_at_offset(26) {
        assert_eq!(s.string, "allo");
    } else {
        panic!("wrong enum type");
    }
}
