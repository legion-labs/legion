mod handler;

use anyhow::{Error, Result};
use handler::MyStreamer;

use legion_online::grpc::Server;
use legion_streaming_proto::streamer_server::StreamerServer;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).map_err::<Error, _>(Into::into)?;

    let service = StreamerServer::new(MyStreamer);

    Server::default()
        .set_listen_address("[::]:5000".parse()?)
        .run(service)
        .await
        .map_err(Into::into)
}
