use crate::{input::Input, output::Output, types::Type};

fn generated_log(input: &Input, output_socket: &Output) {
    // Need to support invalid conversions.
    log(input.get_value().try_into().unwrap());
    output_socket.signal();
}

// #[define(Script)]
fn log(data: String) {
    println!("{}", data);
}

fn generated_add(a: &Input, b: &Input, output: &mut Output) {
    let ret = add(
        a.get_value().try_into().unwrap(),
        b.get_value().try_into().unwrap(),
    );

    output.value = Type::from(ret);
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
