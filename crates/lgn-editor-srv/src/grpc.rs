use std::sync::Arc;

use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    RedoTransactionRequest, RedoTransactionResponse, UndoTransactionRequest,
    UndoTransactionResponse,
};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified
    pub(crate) fn new(transaction_manager: Arc<Mutex<TransactionManager>>) -> Self {
        Self {
            transaction_manager,
        }
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
        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager
            .undo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Undo transaction failed: {}", err)))?;

        Ok(Response::new(UndoTransactionResponse { id: 0 }))
    }

    async fn redo_transaction(
        &self,
        _request: Request<RedoTransactionRequest>,
    ) -> Result<Response<RedoTransactionResponse>, Status> {
        let mut transaction_manager = self.transaction_manager.lock().await;
        transaction_manager
            .redo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Redo transaction failed: {}", err)))?;

        Ok(Response::new(RedoTransactionResponse { id: 0 }))
    }
}
