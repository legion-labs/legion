use std::sync::Arc;

use lgn_async::receiver::SharedUnboundedReceiver;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_data_transaction::TransactionManager;
use lgn_editor_proto::editor::{
    editor_server::{Editor, EditorServer},
    init_log_stream_response, init_message_stream_response, InitLogStreamRequest,
    InitLogStreamResponse, InitMessageStreamRequest, InitMessageStreamResponse,
    RedoTransactionRequest, RedoTransactionResponse, UndoTransactionRequest,
    UndoTransactionResponse,
};
use tokio::sync::{broadcast::error::RecvError, mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};

use crate::broadcast_sink::TraceEvent;

#[derive(Debug, Clone)]
pub(crate) enum EditorEvent {
    SelectionChanged(Vec<ResourceTypeAndId>),
    ResourceChanged(Vec<ResourceTypeAndId>),
}

pub(crate) type TraceEventsReceiver = SharedUnboundedReceiver<TraceEvent>;
pub(crate) type EditorEventsReceiver = SharedUnboundedReceiver<EditorEvent>;

pub(crate) struct GRPCServer {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    /// A globally share trace events, unbounded, receiver
    trace_events_receiver: TraceEventsReceiver,
    editor_events_receiver: EditorEventsReceiver,
}

impl GRPCServer {
    /// Instantiate a new `GRPCServer`
    pub(crate) fn new(
        transaction_manager: Arc<Mutex<TransactionManager>>,
        trace_events_receiver: TraceEventsReceiver,
        editor_events_receiver: EditorEventsReceiver,
    ) -> Self {
        Self {
            transaction_manager,
            trace_events_receiver,
            editor_events_receiver,
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
        let receiver = self.editor_events_receiver.clone();

        tokio::spawn(async move {
            loop {
                match receiver.lock().await.recv().await {
                    Ok(editor_event) => {
                        if let Some(message) = match editor_event {
                            EditorEvent::SelectionChanged(selections) => {
                                Some(lgn_editor_proto::editor::Message {
                                    msg_type:
                                        lgn_editor_proto::editor::MessageType::SelectionChanged
                                            as i32,
                                    payload: serde_json::json!(selections).to_string(),
                                })
                            }
                            EditorEvent::ResourceChanged(changed_resources) => {
                                Some(lgn_editor_proto::editor::Message {
                                    msg_type: lgn_editor_proto::editor::MessageType::ResourceChanged
                                        as i32,
                                    payload: serde_json::json!(changed_resources).to_string(),
                                })
                            }
                        } {
                            if let Err(_error) = tx.send(Ok(InitMessageStreamResponse {
                                response: Some(init_message_stream_response::Response::Message(
                                    message,
                                )),
                            })) {
                                // Sent errors are always related to closed connection:
                                // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
                                // So we can stop the task
                                return;
                            }
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
