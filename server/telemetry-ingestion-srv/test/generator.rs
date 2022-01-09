//! Generator test

use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();
    info!("hello from generator");
    metric_int!("ticks", "Frame Time", 1000);
    metric_float!("ticks", "Frame Time", 1.0);
    Ok(())
}
