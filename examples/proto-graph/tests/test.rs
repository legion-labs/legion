use proto_graph::runtime::{
    input::Input,
    node::{GeneratedLog, Node},
    output::Output,
    types::Type,
};

#[test]
fn test_log() {
    let input = Input::new(Type::String("Hello World!".to_string()));
    let mut node: Box<dyn Node> = Box::new(GeneratedLog::new(input, Output::default()));
    node.run();
}

#[test]
fn test_log_int() {
    let input = Input::new(Type::Int(42));
    let mut node: Box<dyn Node> = Box::new(GeneratedLog::new(input, Output::default()));
    node.run();
}
