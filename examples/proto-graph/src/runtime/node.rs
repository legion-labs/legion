use std::sync::Arc;

use super::{socket::Socket, types::Type};

pub trait Node {
    fn run(&mut self);
}

pub struct GeneratedLog {
    data: Socket,
    output: Socket,
}

impl GeneratedLog {
    pub fn new(data: Socket, output: Socket) -> Self {
        Self { data, output }
    }
}

impl Node for GeneratedLog {
    fn run(&mut self) {
        // Need to support invalid conversions.
        log(&self.data.get_value().try_into().unwrap());
    }
}

// #[define(Script)]
fn log(data: &String) {
    println!("{}", data);
}

pub struct GeneratedAdd {
    a: Socket,
    b: Socket,
    result: Socket,
}

impl GeneratedAdd {
    pub fn new(a: Socket, b: Socket, result: Socket) -> Arc<Box<Self>> {
        Arc::new(Box::new(Self { a, b, result }))
    }
}

impl Node for GeneratedAdd {
    fn run(&mut self) {
        // Need to support invalid conversions.
        let result = Type::from(add(
            self.a.get_value().try_into().unwrap(),
            self.b.get_value().try_into().unwrap(),
        ));

        //*self.result.value.lock().unwrap() = result;
    }
}

// #[define(Script)]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

/*
// public double WaitTime;
// public double TriggeredTime;
// public Entity Output;

pub fn wait(wait_time: &mut Duration, delta_time: &Duration, output_socket: &Output) {
    let result = wait_time.as_nanos() + delta_time.as_nanos();
    // ...
}
*/
