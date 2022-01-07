use lgn_streaming_proto::{
    streamer_server::Streamer, InitializeStreamRequest, InitializeStreamResponse,
};
use lgn_telemetry::info;
use tonic::{Request, Response, Status};

pub struct MyStreamer;

#[tonic::async_trait]
impl Streamer for MyStreamer {
    async fn initialize_stream(
        &self,
        request: Request<InitializeStreamRequest>,
    ) -> Result<Response<InitializeStreamResponse>, Status> {
        let request = request.into_inner();
        info!("gRPC request received: {:?}", request);
        let response = InitializeStreamResponse {
            rtc_session_description: request.rtc_session_description,
            error: "".to_string(),
        };
        Ok(Response::new(response))
    }
}
