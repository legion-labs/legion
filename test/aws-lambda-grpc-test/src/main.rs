//! lambda code test, going to be moved once we use it

mod handler;

use anyhow::Result;
use handler::MyStreamer;
use lgn_online::grpc::Server;
use lgn_streaming_proto::streamer_server::StreamerServer;
use lgn_telemetry_sink::TelemetryGuard;

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();

    let service = StreamerServer::new(MyStreamer);

    Server::default()
        .set_listen_address("[::]:5000".parse()?)
        .run(service)
        .await
        .map_err(Into::into)
}
