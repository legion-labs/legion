use lambda_http::handler;
use legion_aws::lambda::run_lambda;

mod handler;

use handler::MyStreamer;
use legion_online::grpc::AwsLambdaHandler;

use legion_streaming_proto::streamer_server::StreamerServer;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    simple_logger::init_with_level(log::Level::Info)?;

    let server = StreamerServer::new(MyStreamer);
    let server = tonic_web::enable(server);
    let h = AwsLambdaHandler::new(server);
    run_lambda(handler(h)).await
}
