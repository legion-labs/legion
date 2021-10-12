use legion_data_offline::data_container::OfflineDataContainer;
//use tonic::{Request, Response, Status};

use legion_editor_proto::{GetResourcePropertiesResponse, ResourceDescription, ResourceProperty};

pub fn _get_resource_properties_response<T: OfflineDataContainer>(data_container: &T) {
    if let Ok(results) = data_container.get_editor_properties() {
        let _response = GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: "123412342314".into(),
                path: "test.asset".into(),
                version: 1,
            }),
            properties: results
                .iter()
                .map(|reflection_data| ResourceProperty {
                    name: reflection_data.name.into(),
                    r#type: reflection_data.type_name.into(),
                    default_value: reflection_data.default_value.clone(),
                    value: reflection_data.value.clone(),
                    group: reflection_data.group.clone(),
                })
                .collect(),
        };
    }
}

/*
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

    /// Start the `gRPC` server on the specified `addr`.
    pub async fn listen_and_serve(self, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
        info!("gRPC server started and listening on {}.", addr);

        match tonic::transport::Server::builder()
            .add_service(StreamerServer::new(self))
            .serve(addr)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("gRPC server stopped and no longer listening ({})", e);

                Err(e)
            }
        }
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
*/
