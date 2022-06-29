use super::{input::Input, output::Output, types::Type};

pub trait Node {
    fn run(&mut self);
}

pub struct GeneratedLog {
    data: Input,
    output: Output,
}

impl GeneratedLog {
    pub fn new(data: Input, output: Output) -> Self {
        Self { data, output }
    }
}

impl Node for GeneratedLog {
    fn run(&mut self) {
        // Need to support invalid conversions.
        log(&self.data.get_value().try_into().unwrap());
        self.output.signal();
    }
}

// #[define(Script)]
fn log(data: &String) {
    println!("{}", data);
}

struct GeneratedAdd<'a> {
    a: &'a Input,
    b: &'a Input,
    ret: &'a mut Output,
}

impl<'a> Node for GeneratedAdd<'a> {
    fn run(&mut self) {
        // Need to support invalid conversions.
        self.ret.value = Type::from(add(
            self.a.get_value().try_into().unwrap(),
            self.b.get_value().try_into().unwrap(),
        ));
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
