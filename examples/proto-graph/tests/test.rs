use std::sync::Arc;

use proto_graph::runtime::{
    edge::Edge,
    node::{GeneratedAdd, GeneratedLog, Node},
    socket::Socket,
    types::Type,
};

#[test]
fn test_log() {
    let input = Socket::new(Type::String("Hello World!".to_string()));
    let output = Socket::default();
    let mut node: Box<dyn Node> = Box::new(GeneratedLog::new(input, output));
    node.run();
}

#[test]
fn test_log_int() {
    let input = Socket::new(Type::Int(42));
    let mut output = Socket::default();
    let mut node: Box<dyn Node> = Box::new(GeneratedLog::new(input, output));
    node.run();
}

#[test]
fn test_edge() {
    /*
    let log_input = Input::new(Type::String("Hello World!".to_string()));
    let mut log_output = Output::default();
    let mut node_log: Box<dyn Node> = Box::new(GeneratedLog::new(&log_input, &mut log_output));

    let a = Input::new(Type::Float(3.));
    let b = Input::new(Type::Float(4.));
    let mut add_result = Output::default();
    let node_add: Box<dyn Node> = Box::new(GeneratedAdd::new(&a, &b, &mut add_result));

    let edge = Edge::new(&add_result, &log_input);

    node_log.run();
    */
}
