use std::sync::Arc;

use lgn_data_transaction::DataManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    RedoTransactionRequest, RedoTransactionResponse, UndoTransactionRequest,
    UndoTransactionResponse,
};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub(crate) struct GRPCServer {
    data_manager: Arc<Mutex<DataManager>>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified
    /// `webrtc::WebRTCServer`.
    pub(crate) fn new(data_manager: Arc<Mutex<DataManager>>) -> Self {
        Self { data_manager }
    }

    pub fn service(self) -> EditorServer<Self> {
        EditorServer::new(self)
    }
}

#[tonic::async_trait]
impl Editor for GRPCServer {
    async fn undo_transaction(
        &self,
        _request: Request<UndoTransactionRequest>,
    ) -> Result<Response<UndoTransactionResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .undo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Undo transaction failed: {}", err)))?;

        Ok(Response::new(UndoTransactionResponse { id: 0 }))
    }

    async fn redo_transaction(
        &self,
        _request: Request<RedoTransactionRequest>,
    ) -> Result<Response<RedoTransactionResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .redo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Redo transaction failed: {}", err)))?;

        Ok(Response::new(RedoTransactionResponse { id: 0 }))
    }
}
