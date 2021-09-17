use transit::*;

#[derive(TransitReflect)]
struct MyTestEvent {
    some_64: u64,
    some_32: u32,
}

#[derive(TransitReflect)]
struct OtherEvent {
    some_64: u64,
}

declare_queue_struct!(
    struct MyQueue<MyTestEvent, OtherEvent> {}
);

#[test]
fn test_queue() {
    let q = MyQueue::new(1024);
    q.push_my_test_event(MyTestEvent {
        some_64: 0,
        some_32: 1,
    });
    q.push_other_event(OtherEvent { some_64: 0 });
}
