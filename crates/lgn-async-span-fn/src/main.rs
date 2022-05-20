//! Dumb binary to test async span fn

use std::time::Duration;

use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::span_fn;
use tokio::time::sleep;

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

// // Generated
// async fn delayed_() {
//     lgn_tracing::async_span_scope!(_METADATA_FUNC, concat!(module_path!(), "::", "delayed"));

//     println!("Before");

//     {
//         lgn_tracing::async_span_scope!(_METADATA_AWAIT_0, concat!(module_path!(), "::", "delayed"));
//         sleep(Duration::from_millis(10)).await;
//     };

//     println!("Second");

//     let msg = {
//         lgn_tracing::async_span_scope!(_METADATA_AWAIT_1, concat!(module_path!(), "::", "delayed"));
//         let result = delayed_value().await;
//         result
//     };

//     println!("{}", msg);
// }

// // Generated
// async fn delayed_() {
//     lgn_tracing::async_span_scope!(_METADATA_FUNC, concat!(module_path!(), "::", "delayed"));
//     println!("Before");
//     lgn_tracing::async_span_scope!(
//         _METADATA_AWAIT_0_BEGIN,
//         concat!(module_path!(), "::", "delayed")
//     );
//     sleep(Duration::from_millis(10)).await;
//     lgn_tracing::async_span_scope!(
//         _METADATA_AWAIT_0_END,
//         concat!(module_path!(), "::", "delayed")
//     );
//     println!("Second");
//     lgn_tracing::async_span_scope!(
//         _METADATA_AWAIT_1_BEGIN,
//         concat!(module_path!(), "::", "delayed")
//     );
//     sleep(Duration::from_millis(10)).await;
//     lgn_tracing::async_span_scope!(
//         _METADATA_AWAIT_1_END,
//         concat!(module_path!(), "::", "delayed")
//     );
//     println!("After");
// }

#[tokio::main]
async fn main() {
    let _telemetry_guard = TelemetryGuard::default().unwrap();

    delayed().await;
}
