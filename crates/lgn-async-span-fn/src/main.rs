//! Dumb binary to test async span fn

use std::time::Duration;

use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::span_fn;
use tokio::time::sleep;

#[span_fn]
async fn iteration_with_cond() {
    let a = 3;

    loop {
        if a == 3 {
            println!("a was 3");
            sleep(Duration::from_millis(10)).await;
        }

        break;
    }
}

#[span_fn]
async fn delayed_value() -> String {
    sleep(Duration::from_millis(10)).await;

    "After".into()
}

#[span_fn]
async fn delayed() {
    println!("Before");

    sleep(Duration::from_millis(10)).await;

    println!("Second");

    let msg = delayed_value().await;

    println!("{}", msg);
}

#[tokio::main]
async fn main() {
    let _telemetry_guard = TelemetryGuard::default().unwrap();

    delayed().await;

    iteration_with_cond().await;
}
