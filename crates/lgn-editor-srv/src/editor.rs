use std::sync::Arc;

use async_trait::async_trait;
use lgn_data_transaction::TransactionManager;
use lgn_editor_yaml::editor::{
    requests::{self},
    responses::{self, RedoTransactionResponse, UndoTransactionResponse},
    Api,
};
use lgn_online::server::Result;
use tokio::sync::Mutex;

pub(crate) struct Server {
    transaction_manager: Arc<Mutex<TransactionManager>>,
}

impl Server {
    pub(crate) fn new(transaction_manager: Arc<Mutex<TransactionManager>>) -> Self {
        Self {
            transaction_manager,
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn undo_transaction(
        &self,
        _parts: http::request::Parts,
        _request: requests::UndoTransactionRequest,
    ) -> Result<responses::UndoTransactionResponse> {
        let mut transaction_manager = self.transaction_manager.lock().await;

        match transaction_manager.undo_transaction().await {
            Ok(_) => Ok(UndoTransactionResponse::Status204),
            Err(_) => Ok(UndoTransactionResponse::Status409),
        }
    }

    async fn redo_transaction(
        &self,
        _parts: http::request::Parts,
        _request: requests::RedoTransactionRequest,
    ) -> Result<responses::RedoTransactionResponse> {
        let mut transaction_manager = self.transaction_manager.lock().await;

        match transaction_manager.redo_transaction().await {
            Ok(_) => Ok(RedoTransactionResponse::Status204),
            Err(_) => Ok(RedoTransactionResponse::Status409),
        }
    }

    /*
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
    */
}
