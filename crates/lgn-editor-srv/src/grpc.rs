use std::{ops::Deref, sync::Arc};

use lgn_data_runtime::ResourceTypeAndId;
use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    init_log_stream_response, init_message_stream_response, InitLogStreamRequest,
    InitLogStreamResponse, InitMessageStreamRequest, InitMessageStreamResponse,
    RedoTransactionRequest, RedoTransactionResponse, UndoTransactionRequest,
    UndoTransactionResponse,
};
use tokio::sync::{
    broadcast::{self, error::RecvError},
    mpsc, Mutex,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};

use crate::broadcast_sink::TraceEvent;

/// Easy to share, referenced counted version of Tokio's [`broadcast::Receiver`].
/// Can be cloned safely and will dereference to the internal [`Mutex`].
pub(crate) struct SharedUnboundedReceiver<T>(pub(crate) Arc<Mutex<broadcast::Receiver<T>>>);

impl<T> SharedUnboundedReceiver<T> {
    pub fn new(receiver: broadcast::Receiver<T>) -> Self {
        Self(Arc::new(Mutex::new(receiver)))
    }
}

impl<T> Clone for SharedUnboundedReceiver<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for SharedUnboundedReceiver<T> {
    type Target = Mutex<broadcast::Receiver<T>>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> From<broadcast::Receiver<T>> for SharedUnboundedReceiver<T> {
    fn from(receiver: broadcast::Receiver<T>) -> Self {
        Self::new(receiver)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SelectionEvent {
    SelectionChanged(Vec<ResourceTypeAndId>),
}

pub(crate) type TraceEventsReceiver = SharedUnboundedReceiver<TraceEvent>;
pub(crate) type SelectionEventsReceiver = SharedUnboundedReceiver<SelectionEvent>;

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    /// A globally share trace events, unbounded, receiver
    trace_events_receiver: TraceEventsReceiver,
    selection_events_receiver: SelectionEventsReceiver,
}

impl GRPCServer {
    /// Instantiate a new `GRPCServer`
    pub(crate) fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        trace_events_receiver: TraceEventsReceiver,
        selection_events_receiver: SelectionEventsReceiver,
    ) -> Self {
        Self {
            transaction_manager,
            trace_events_receiver,
            selection_events_receiver,
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

    type InitLogStreamStream = UnboundedReceiverStream<Result<InitLogStreamResponse, Status>>;

    async fn init_log_stream(
        &self,
        _: Request<InitLogStreamRequest>,
    ) -> Result<tonic::Response<<Self as Editor>::InitLogStreamStream>, Status> {
        let (tx, rx) = mpsc::unbounded_channel();
        let receiver = self.trace_events_receiver.clone();

        tokio::spawn(async move {
            loop {
                match receiver.lock().await.recv().await {
                    Ok(TraceEvent::Message {
                        target,
                        message,
                        level,
                        time,
                    }) => {
                        if let Err(_error) = tx.send(Ok(InitLogStreamResponse {
                            response: Some(init_log_stream_response::Response::TraceEvent(
                                lgn_editor_proto::editor::TraceEvent {
                                    // There must be a default, zero, value for enums but Level is 1-indexed
                                    // (https://developers.google.com/protocol-buffers/docs/proto3#enum)
                                    // So we simply decrement the level to get the proper value at runtime
                                    level: (level as i32 - 1),
                                    message,
                                    target,
                                    time,
                                },
                            )),
                        })) {
                            // Sent errors are always related to closed connection:
                            // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
                            // So we can stop the task
                            return;
                        }
                    }
                    Ok(_trace_event) => {
                        // Ignoring other events for now
                    }
                    Err(RecvError::Lagged(skipped_messages)) => {
                        if let Err(_error) = tx.send(Ok(InitLogStreamResponse {
                            response: Some(init_log_stream_response::Response::Lagging(
                                skipped_messages,
                            )),
                        })) {
                            // Sent errors are always related to closed connection:
                            // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
                            // So we can stop the task
                            return;
                        }
                    }
                    Err(RecvError::Closed) => return,
                }
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    type InitMessageStreamStream =
        UnboundedReceiverStream<Result<InitMessageStreamResponse, Status>>;

    async fn init_message_stream(
        &self,
        _: Request<InitMessageStreamRequest>,
    ) -> Result<tonic::Response<<Self as Editor>::InitMessageStreamStream>, Status> {
        let (tx, rx) = mpsc::unbounded_channel();
        let receiver = self.selection_events_receiver.clone();

        tokio::spawn(async move {
            loop {
                match receiver.lock().await.recv().await {
                    Ok(selection_event) => {
                        let SelectionEvent::SelectionChanged(selections) = selection_event;

                        if let Err(_error) = tx.send(Ok(InitMessageStreamResponse {
                            response: Some(init_message_stream_response::Response::Message(
                                lgn_editor_proto::editor::Message {
                                    msg_type: 0,
                                    payload: serde_json::json!(selections).to_string(),
                                },
                            )),
                        })) {
                            // Sent errors are always related to closed connection:
                            // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
                            // So we can stop the task
                            return;
                        }
                    }
                    Err(RecvError::Lagged(skipped_messages)) => {
                        if let Err(_error) = tx.send(Ok(InitMessageStreamResponse {
                            response: Some(init_message_stream_response::Response::Lagging(
                                skipped_messages,
                            )),
                        })) {
                            // Sent errors are always related to closed connection:
                            // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
                            // So we can stop the task
                            return;
                        }
                    }
                    Err(RecvError::Closed) => return,
                }
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
