use transit::*;

#[derive(TransitReflect)]
#[allow(dead_code)]
struct MyTestEvent {
    some_64: u64,
    some_32: u32,
}

#[test]
fn test_reflect_simple_struct() {
    let res = MyTestEvent::reflect();
    assert_eq!("MyTestEvent", res.name);
    assert_eq!(16, res.size);
    assert_eq!(2, res.members.len());
    assert_eq!("some_64", res.members[0].name);
    assert_eq!(8, res.members[0].size);
    assert_eq!(0, res.members[0].offset);
    assert_eq!("some_32", res.members[1].name);
    assert_eq!(4, res.members[1].size);
    assert_eq!(8, res.members[1].offset);
}
