use transit::*;

#[derive(TransitReflect, Debug)]
struct MyTestEvent {
    some_64: u64,
    some_32: u32,
}

#[derive(TransitReflect, Debug)]
struct OtherEvent {
    some_64: u64,
}

declare_queue_struct!(
    struct MyQueue<MyTestEvent, OtherEvent> {}
);

#[test]
fn test_queue() {
    let mut q = MyQueue::new(1024);
    q.push_my_test_event(MyTestEvent {
        some_64: 2,
        some_32: 1,
    });
    assert_eq!(17, q.len_bytes());
    q.push_other_event(OtherEvent { some_64: 3 });
    assert_eq!(26, q.len_bytes());
}
