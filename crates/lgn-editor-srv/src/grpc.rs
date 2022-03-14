use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    InitLogsStreamRequest, InitLogsStreamResponse, RedoTransactionRequest, RedoTransactionResponse,
    UndoTransactionRequest, UndoTransactionResponse,
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::channel_sink::Log;

pub(crate) struct LogsReceiver(pub(crate) mpsc::UnboundedReceiver<Log>);

impl LogsReceiver {
    pub fn new(receiver: mpsc::UnboundedReceiver<Log>) -> Self {
        Self(receiver)
    }
}

impl Deref for LogsReceiver {
    type Target = mpsc::UnboundedReceiver<Log>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LogsReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    log_receiver: Arc<Mutex<LogsReceiver>>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified
    pub(crate) fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        log_receiver: Arc<Mutex<LogsReceiver>>,
    ) -> Self {
        Self {
            transaction_manager,
            log_receiver,
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

    type InitLogsStreamStream = ReceiverStream<Result<InitLogsStreamResponse, Status>>;

    async fn init_logs_stream(
        &self,
        _: Request<InitLogsStreamRequest>,
    ) -> Result<tonic::Response<<Self as Editor>::InitLogsStreamStream>, Status> {
        let (tx, rx) = mpsc::channel(10);

        let receiver = self.log_receiver.clone();

        tokio::spawn(async move {
            while let Some(log) = receiver.lock().await.recv().await {
                if let Log::Message {
                    target,
                    message,
                    level,
                    time,
                } = log
                {
                    let _send_result = tx
                        .send(Ok(InitLogsStreamResponse {
                            level: level as i32,
                            message,
                            target,
                            time,
                        }))
                        .await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
