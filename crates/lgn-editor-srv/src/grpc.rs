use std::{ops::Deref, sync::Arc};

use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    InitLogsStreamRequest, InitLogsStreamResponse, RedoTransactionRequest, RedoTransactionResponse,
    UndoTransactionRequest, UndoTransactionResponse,
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::channel_sink::TraceEvent;

/// Easy to share, referenced counted version of Tokio's `UnboundedReceiver`.
/// Can be cloned safely and will dereference to the internal [`Mutex`].
pub(crate) struct SharedUnboundedReceiver<T>(pub(crate) Arc<Mutex<mpsc::UnboundedReceiver<T>>>);

impl<T> SharedUnboundedReceiver<T> {
    pub fn new(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self(Arc::new(Mutex::new(receiver)))
    }
}

impl<T> Clone for SharedUnboundedReceiver<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for SharedUnboundedReceiver<T> {
    type Target = Mutex<mpsc::UnboundedReceiver<T>>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> From<mpsc::UnboundedReceiver<T>> for SharedUnboundedReceiver<T> {
    fn from(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self::new(receiver)
    }
}

pub(crate) type TraceEventsReceiver = SharedUnboundedReceiver<TraceEvent>;

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    trace_events_receiver: TraceEventsReceiver,
}

impl GRPCServer {
    /// Instantiate a new `GRPCServer`
    pub(crate) fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        trace_events_receiver: TraceEventsReceiver,
    ) -> Self {
        Self {
            transaction_manager,
            trace_events_receiver,
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

        let receiver = self.trace_events_receiver.clone();

        tokio::spawn(async move {
            while let Some(trace_event) = receiver.lock().await.recv().await {
                if let TraceEvent::Message {
                    target,
                    message,
                    level,
                    time,
                } = trace_event
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
