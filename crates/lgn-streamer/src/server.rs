use std::sync::Arc;

use async_trait::async_trait;
use lgn_online::server::{Error, Result};
use lgn_tracing::{debug, info};

use crate::{
    api::streaming::{
        server::{InitializeStreamRequest, InitializeStreamResponse},
        Api,
    },
    streamer::StreamEvent,
    webrtc::WebRTCServer,
};

/// The `api` implementation for the streaming server.
pub(crate) struct Server {
    webrtc_server: WebRTCServer,
    stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
}

/// Stream represents an established stream.
impl Server {
    /// Instantiate a new `Server` using the specified
    /// `webrtc::WebRTCServer`.
    pub(crate) fn new(
        webrtc_server: WebRTCServer,
        stream_events_sender: crossbeam::channel::Sender<StreamEvent>,
    ) -> Self {
        Self {
            webrtc_server,
            stream_events_sender: Arc::new(stream_events_sender),
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn initialize_stream(
        &self,
        request: InitializeStreamRequest,
    ) -> Result<InitializeStreamResponse> {
        debug!("Initializing a new WebRTC stream connection...");

        let rtc_session_description = self
            .webrtc_server
            .initialize_stream_connection(request.body.0, Arc::clone(&self.stream_events_sender))
            .await
            .map_err(|err| Error::internal(err.to_string()))?;

        info!("New WebRTC stream connection initialized.");

        Ok(InitializeStreamResponse::Status200(
            rtc_session_description.into(),
        ))
    }
}
