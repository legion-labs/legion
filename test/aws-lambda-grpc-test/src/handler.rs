use lgn_streaming_proto::{
    initialize_stream_response, streamer_server::Streamer, AddIceCandidatesRequest,
    AddIceCandidatesResponse, IceCandidateRequest, IceCandidateResponse, InitializeStreamRequest,
    InitializeStreamResponse,
};
use log::info;
use tokio_stream::wrappers::ReceiverStream;
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
            response: Some(initialize_stream_response::Response::Ok(
                initialize_stream_response::Ok {
                    rtc_session_description: request.rtc_session_description,
                    stream_id: "stream-id".to_string(),
                },
            )),
        };

        Ok(Response::new(response))
    }

    // TODO: Implement test
    async fn add_ice_candidates(
        &self,
        _request: Request<AddIceCandidatesRequest>,
    ) -> Result<Response<AddIceCandidatesResponse>, Status> {
        unimplemented!()
    }

    #[doc = "Server streaming response type for the IceCandidates method."]
    type IceCandidatesStream = ReceiverStream<Result<IceCandidateResponse, Status>>;

    // TODO: Implement test
    async fn ice_candidates(
        &self,
        _request: Request<IceCandidateRequest>,
    ) -> Result<Response<Self::IceCandidatesStream>, Status> {
        unimplemented!()
    }
}
