use std::{ops::Deref, sync::Arc};

use lgn_data_runtime::ResourceTypeAndId;
use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    InitLogStreamRequest, InitLogStreamResponse, InitMessageStreamRequest,
    InitMessageStreamResponse, RedoTransactionRequest, RedoTransactionResponse,
    UndoTransactionRequest, UndoTransactionResponse,
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::channel_sink::TraceEvent;

/// Easy to share, referenced counted version of Tokio's [`mpsc::UnboundedReceiver`].
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

pub(crate) enum SelectionEvent {
    SelectionChanged(Vec<ResourceTypeAndId>),
}

pub(crate) type TraceEventsReceiver = SharedUnboundedReceiver<TraceEvent>;
pub(crate) type SelectionEventsReceiver = SharedUnboundedReceiver<SelectionEvent>;

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    trace_events_receiver: TraceEventsReceiver,
    selection_events: SelectionEventsReceiver,
}

impl GRPCServer {
    /// Instantiate a new `GRPCServer`
    pub(crate) fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        trace_events_receiver: TraceEventsReceiver,
        selection_events: SelectionEventsReceiver,
    ) -> Self {
        Self {
            transaction_manager,
            trace_events_receiver,
            selection_events,
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

    type InitLogStreamStream = ReceiverStream<Result<InitLogStreamResponse, Status>>;

    async fn init_log_stream(
        &self,
        _: Request<InitLogStreamRequest>,
    ) -> Result<tonic::Response<<Self as Editor>::InitLogStreamStream>, Status> {
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
                        .send(Ok(InitLogStreamResponse {
                            // There must be a default, zero, value for enums but Level is 1-indexed
                            // (https://developers.google.com/protocol-buffers/docs/proto3#enum)
                            // So we simply decrement the level to get the proper value at runtime
                            level: (level as i32 - 1),
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

    type InitMessageStreamStream = ReceiverStream<Result<InitMessageStreamResponse, Status>>;

    async fn init_message_stream(
        &self,
        _: Request<InitMessageStreamRequest>,
    ) -> Result<tonic::Response<<Self as Editor>::InitMessageStreamStream>, Status> {
        let (tx, rx) = mpsc::channel(1);
        let receiver = self.selection_events.clone();
        tokio::spawn(async move {
            while let Some(selection_event) = receiver.lock().await.recv().await {
                let SelectionEvent::SelectionChanged(selections) = selection_event;
                let _send_result = tx
                    .send(Ok(InitMessageStreamResponse {
                        msg_type: 0,
                        payload: serde_json::json!(selections).to_string(),
                    }))
                    .await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
