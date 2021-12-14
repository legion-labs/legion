mod handler;

use anyhow::Result;
use handler::MyStreamer;
use lgn_online::grpc::Server;
use lgn_streaming_proto::streamer_server::StreamerServer;

#[tokio::main]
async fn main() -> Result<()> {
    lgn_logger::Logger::init(lgn_logger::Config::default()).unwrap();

    let service = StreamerServer::new(MyStreamer);

    Server::default()
        .set_listen_address("[::]:5000".parse()?)
        .run(service)
        .await
        .map_err(Into::into)
}
