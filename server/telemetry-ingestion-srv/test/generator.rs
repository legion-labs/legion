use std::sync::Arc;
use telemetry::*;

fn init_telemetry() {
    let sink = Arc::new(GRPCEventSink::new("http://127.0.0.1:8080"));
    init_event_dispatch(1024, 1024 * 1024, sink).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();
    shutdown_event_dispatch();
    Ok(())
}
