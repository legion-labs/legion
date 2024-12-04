//! Dumb binary to test async span fn

#![allow(clippy::never_loop)]

use std::time::Duration;

use futures::future::join_all;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{info, span_fn};
use tokio::{fs::File, io::AsyncReadExt, time::sleep};

#[span_fn]
async fn empty_return() {
    sleep(Duration::from_millis(1)).await;
}

#[span_fn]
async fn iteration_with_cond() {
    let a = 3;

    loop {
        if a == 3 {
            println!("a was 3");
            sleep(Duration::from_millis(1)).await;
        }

        break;
    }

    info!("inside my_function!");
}

#[span_fn]
async fn delayed_value() -> String {
    sleep(Duration::from_millis(1)).await;

    let msg = "After".into();

    sleep(Duration::from_millis(1)).await;

    msg
}

#[span_fn]
fn consume_delayed_value(_: String) {
    println!("Consumed a delayed value");
}

#[span_fn]
async fn delayed() {
    println!("First");

    sleep(Duration::from_millis(1)).await;

    println!("Second");

    let msg = delayed_value().await;

    println!("{}", msg);

    consume_delayed_value(delayed_value().await);
}

#[span_fn]
async fn read_txt() {
    delayed().await;

    let mut file = File::open("./test.txt").await.unwrap();

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let _len = file.metadata().await.unwrap().len();
}

#[span_fn]
async fn read_all_txt() {
    let mut counter = 0;

    let mut futures = Vec::new();

    while counter < 3 {
        let handle = async move {
            read_txt().await;
        };

        futures.push(handle);

        counter += 1;
    }

    join_all(futures).await;
}

#[tokio::main]
async fn main() {
    let _telemetry_guard = TelemetryGuard::default().unwrap();

    delayed_value().await;
    delayed_value().await;

    read_txt().await;

    delayed().await;

    iteration_with_cond().await;

    read_all_txt().await;

    empty_return().await;
}
