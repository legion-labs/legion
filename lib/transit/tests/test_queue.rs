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

#[allow(dead_code)]
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
        some_32: 1,
    });
    assert_eq!(17, q.len_bytes());
    q.push_other_event(OtherEvent { some_64: 3 });
    assert_eq!(26, q.len_bytes());

    q.push_dyn_string(DynString {
        string: String::from("allo"),
    });
    assert_eq!(35, q.len_bytes());
}
