use std::sync::Arc;

use lgn_streaming_proto::{
    initialize_stream_response,
    streamer_server::{Streamer, StreamerServer},
    AddIceCandidatesRequest, AddIceCandidatesResponse, IceCandidateRequest, IceCandidateResponse,
    InitializeStreamRequest, InitializeStreamResponse,
};
use lgn_utils::StableHashMap;
use log::{debug, info, warn};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use webrtc::peer::{ice::ice_candidate::RTCIceCandidate, peer_connection::RTCPeerConnection};

use crate::{streamer::StreamEvent, webrtc::WebRTCServer};

/// The `gRPC` server implementation for the streaming server.
pub(crate) struct GRPCServer {
    webrtc_server: WebRTCServer,
    stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
    // TODO: Peer connection doesn't have to be an Arc
    rtc_peer_connnections: Arc<Mutex<StableHashMap<String, Arc<RTCPeerConnection>>>>,
    stream_ice_candidates_senders:
        Arc<Mutex<StableHashMap<String, Arc<crossbeam::channel::Sender<RTCIceCandidate>>>>>,
    stream_ice_candidates_receivers:
        Arc<Mutex<StableHashMap<String, Arc<crossbeam::channel::Receiver<RTCIceCandidate>>>>>,
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
            rtc_peer_connnections: Arc::new(Mutex::new(StableHashMap::from_iter(Vec::new()))),
            stream_ice_candidates_senders: Arc::new(Mutex::new(StableHashMap::from_iter(
                Vec::new(),
            ))),
            stream_ice_candidates_receivers: Arc::new(Mutex::new(StableHashMap::from_iter(
                Vec::new(),
            ))),
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

        // This channel is in charge to push the new ice candidates to the clients and vice versa
        let (stream_ice_candidates_sender, stream_ice_candidates_receiver) =
            crossbeam::channel::unbounded();

        let stream_ice_candidates_sender = Arc::new(stream_ice_candidates_sender);

        let remote_description = self
            .webrtc_server
            .initialize_stream_connection(
                request.into_inner().rtc_session_description,
                Arc::clone(&self.stream_events_sender),
                Arc::clone(&stream_ice_candidates_sender),
            )
            .await;

        Ok(Response::new(match remote_description {
            Ok((stream_id, peer_connection, rtc_session_description)) => {
                info!("New WebRTC stream connection initialized.");

                self.stream_ice_candidates_senders
                    .lock()
                    .await
                    .insert(stream_id.to_string(), stream_ice_candidates_sender);

                self.stream_ice_candidates_receivers.lock().await.insert(
                    stream_id.to_string(),
                    Arc::new(stream_ice_candidates_receiver),
                );

                self.rtc_peer_connnections
                    .lock()
                    .await
                    .insert(stream_id.to_string(), peer_connection);

                InitializeStreamResponse {
                    response: Some(initialize_stream_response::Response::Ok(
                        initialize_stream_response::Ok {
                            rtc_session_description,
                            stream_id: stream_id.to_string(),
                        },
                    )),
                }
            }
            Err(error) => {
                warn!(
                    "Failed to initialize new WebRTC stream connection: {}.",
                    error
                );

                InitializeStreamResponse {
                    response: Some(initialize_stream_response::Response::Error(
                        error.to_string(),
                    )),
                }
            }
        }))
    }

    async fn add_ice_candidates(
        &self,
        request: Request<AddIceCandidatesRequest>,
    ) -> Result<Response<AddIceCandidatesResponse>, Status> {
        let AddIceCandidatesRequest {
            stream_id,
            ice_candidates,
        } = request.into_inner();

        let rtc_peer_connections = self.rtc_peer_connnections.lock().await;

        let peer_connection = rtc_peer_connections.get(&stream_id);

        let ok = if let Some(peer_connection) = peer_connection {
            for ice_candidate in ice_candidates {
                let ice_candidate = serde_json::from_slice(&ice_candidate)
                    .map_err(|error| Status::invalid_argument(error.to_string()))?;

                peer_connection
                    .add_ice_candidate(ice_candidate)
                    .await
                    .map_err(|error| {
                        Status::internal(format!(
                            "Couldn't add ice candidate to peer connection: {}",
                            error.to_string(),
                        ))
                    })?;
            }

            true
        } else {
            false
        };

        Ok(Response::new(AddIceCandidatesResponse { ok }))
    }

    #[doc = "Server streaming response type for the IceCandidates method."]
    type IceCandidatesStream = ReceiverStream<Result<IceCandidateResponse, Status>>;

    async fn ice_candidates(
        &self,
        request: Request<IceCandidateRequest>,
    ) -> Result<Response<Self::IceCandidatesStream>, Status> {
        let ice_candidate_request = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::channel(4);

        // TODO: Super ugly code below: cleanup
        let stream_ice_candidates_receivers = &self.stream_ice_candidates_receivers.lock().await;

        let stream_ice_candidates_receiver =
            stream_ice_candidates_receivers.get(&ice_candidate_request.stream_id);

        if let Some(stream_ice_candidates_receiver) = stream_ice_candidates_receiver {
            let stream_ice_candidates_receiver = Arc::clone(stream_ice_candidates_receiver);

            tokio::spawn(async move {
                for ice_candidate in stream_ice_candidates_receiver
                    .iter()
                    .collect::<Vec<RTCIceCandidate>>()
                {
                    tx.send(Ok(IceCandidateResponse {
                        ice_candidate: serde_json::to_vec(&ice_candidate).unwrap(),
                    }))
                    .await
                    .unwrap();
                }
            });

            return Ok(Response::new(ReceiverStream::new(rx)));
        }

        Err(Status::failed_precondition("Stream id not found"))
    }
}
