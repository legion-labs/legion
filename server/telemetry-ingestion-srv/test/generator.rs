use lgn_telemetry::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetrySystemGuard::new();
    log_static_str(LogLevel::Info, "hello from generator");
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Frame Time",
        unit: "ticks",
    };
    record_int_metric(&FRAME_TIME_METRIC, 1000);
    record_float_metric(&FRAME_TIME_METRIC, 1.0);
    Ok(())
}
