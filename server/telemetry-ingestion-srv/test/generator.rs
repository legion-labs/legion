use telemetry::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();
    log_str(LogLevel::Info, "hello from generator");
    shutdown_telemetry();
    Ok(())
}
