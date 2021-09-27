use telemetry::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();
    shutdown_telemetry();
    Ok(())
}
