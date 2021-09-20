use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use tonic::{Request, Response, Status};

use legion_editor_proto::{editor_server::*, InitializeStreamRequest, InitializeStreamResponse};

use super::webrtc::WebRTCServer;

// Implement the editor server logic.
pub struct Server {
    webrtc_server: Arc<Mutex<WebRTCServer>>,
}

impl Server {
    pub fn new() -> Result<Self, anyhow::Error> {
        let webrtc_server = Arc::new(Mutex::new(WebRTCServer::new()?));

        Ok(Server { webrtc_server })
    }

    pub async fn listen_and_serve(
        addr: SocketAddr,
        srv: Self,
    ) -> Result<(), tonic::transport::Error> {
        tonic::transport::Server::builder()
            .add_service(EditorServer::new(srv))
            .serve(addr)
            .await
    }
}

#[tonic::async_trait]
impl Editor for Server {
    async fn initialize_stream(
        &self,
        request: Request<InitializeStreamRequest>,
    ) -> Result<Response<InitializeStreamResponse>, Status> {
        let webrtc_server = self.webrtc_server.lock().await;
        let rtc_session_description = webrtc_server
            .initialize_stream(request.into_inner().rtc_session_description)
            .await;

        Ok(Response::new(match rtc_session_description {
            Ok(rtc_session_description) => InitializeStreamResponse {
                rtc_session_description,
                error: String::default(),
            },
            Err(e) => InitializeStreamResponse {
                rtc_session_description: vec![],
                error: format!("{}", e),
            },
        }))
    }
}
