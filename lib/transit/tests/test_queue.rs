use lgn_transit::prelude::*;

#[derive(Debug, TransitReflect)]
pub struct MyTestEvent {
    some_64: u64,
    some_32: u32,
}

impl InProcSerialize for MyTestEvent {}

#[derive(Debug, TransitReflect)]
pub struct OtherEvent {
    some_64: u64,
}

impl InProcSerialize for OtherEvent {}

declare_queue_struct!(
    struct MyQueue<MyTestEvent, OtherEvent> {}
);

#[test]
fn test_queue() {
    assert!(<MyTestEvent as InProcSerialize>::is_size_static());
    assert!(<OtherEvent as InProcSerialize>::is_size_static());

    let mut q = MyQueue::new(1024);
    q.push(MyTestEvent {
        some_64: 2,
        some_32: 3,
    });
    assert_eq!(17, q.len_bytes());

    q.push(OtherEvent { some_64: 3 });
    assert_eq!(26, q.len_bytes());

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
}
