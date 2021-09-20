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

#[derive(Debug)]
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

struct MyQueueIterator<'a> {
    queue: &'a MyQueue,
    offset: usize,
}

impl core::iter::Iterator for MyQueueIterator<'_> {
    type Item = MyQueueAny;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.queue.len_bytes() {
            None
        } else {
            let (obj, next_offset) = self.queue.read_value_at_offset(self.offset);
            self.offset = next_offset;
            Some(obj)
        }
    }
}

impl MyQueue {
    pub fn iter(&self) -> MyQueueIterator {
        MyQueueIterator {
            queue: self,
            offset: 0,
        }
    }
}

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

    if let (MyQueueAny::MyTestEvent(e), next_obj_offset) = q.read_value_at_offset(0) {
        assert_eq!(e.some_64, 2);
        assert_eq!(e.some_32, 3);
        assert_eq!(next_obj_offset, 17);
    } else {
        panic!("wrong enum type");
    }

    if let (MyQueueAny::OtherEvent(e), next_obj_offset) = q.read_value_at_offset(17) {
        assert_eq!(e.some_64, 3);
        assert_eq!(next_obj_offset, 26);
    } else {
        panic!("wrong enum type");
    }

    if let (MyQueueAny::DynString(s), next_obj_offset) = q.read_value_at_offset(26) {
        assert_eq!(s.string, "allo");
        assert_eq!(next_obj_offset, 35);
    } else {
        panic!("wrong enum type");
    }

    for x in q.iter() {
        dbg!(x);
    }
}
