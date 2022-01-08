//! Generator test

use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();
    info!("hello from generator");
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Frame Time",
        unit: "ticks",
    };
    record_int_metric(&FRAME_TIME_METRIC, 1000);
    record_float_metric(&FRAME_TIME_METRIC, 1.0);
    Ok(())
}
