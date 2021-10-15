use crate::{streamer::StreamEvent, webrtc::WebRTCServer};

use log::{debug, info, warn};
use std::sync::Arc;

use tonic::{Request, Response, Status};

use legion_streaming_proto::{
    streamer_server::{Streamer, StreamerServer},
    InitializeStreamRequest, InitializeStreamResponse,
};

/// The `gRPC` server implementation for the streaming server.
pub(crate) struct GRPCServer {
    webrtc_server: WebRTCServer,
    stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
}

/// Stream represents an established stream.
impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new(
        webrtc_server: WebRTCServer,
        stream_events_sender: crossbeam::channel::Sender<StreamEvent>,
    ) -> Self {
        Self {
            webrtc_server,
            stream_events_sender: Arc::new(stream_events_sender),
        }
    }

    pub fn service(self) -> StreamerServer<Self> {
        StreamerServer::new(self)
    }
}

#[tonic::async_trait]
impl Streamer for GRPCServer {
    async fn initialize_stream(
        &self,
        request: Request<InitializeStreamRequest>,
    ) -> Result<Response<InitializeStreamResponse>, Status> {
        debug!("Initializing a new WebRTC stream connection...");

        let remote_description = self
            .webrtc_server
            .initialize_stream_connection(
                request.into_inner().rtc_session_description,
                Arc::clone(&self.stream_events_sender),
            )
            .await;

        Ok(Response::new(match remote_description {
            Ok(rtc_session_description) => {
                info!("New WebRTC stream connection initialized.");

                InitializeStreamResponse {
                    rtc_session_description,
                    error: String::default(),
                }
            }
            Err(e) => {
                warn!("Failed to initialize new WebRTC stream connection: {}.", e);

                InitializeStreamResponse {
                    rtc_session_description: vec![],
                    error: format!("{}", e),
                }
            }
        }))
    }
}
