use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use editor_srv::editor::Api;
use editor_srv::editor::{
    server::{
        GetMessagesRequest, GetMessagesResponse, RedoTransactionRequest, RedoTransactionResponse,
        UndoTransactionRequest, UndoTransactionResponse,
    },
    Message, MessageMsgType,
};
use lgn_data_transaction::TransactionManager;
use lgn_online::server::{Error, Result};
use tokio::{sync::broadcast::error::RecvError, sync::Mutex, time::sleep};

use crate::grpc::{EditorEvent, EditorEventsReceiver};

pub(crate) struct Server {
    transaction_manager: Arc<Mutex<TransactionManager>>,
    editor_events_receiver: EditorEventsReceiver,
}
/*
impl Server {
    pub(crate) fn new(transaction_manager: Arc<Mutex<TransactionManager>>, editor_events_receiver: EditorEventsReceiver) -> Self {
        Self {
            transaction_manager,
            editor_events_receiver,
        }
    }
}
*/

#[async_trait]
impl Api for Server {
    async fn undo_transaction(
        &self,
        _request: UndoTransactionRequest,
    ) -> Result<UndoTransactionResponse> {
        let mut transaction_manager = self.transaction_manager.lock().await;

        match transaction_manager.undo_transaction().await {
            Ok(_) => Ok(UndoTransactionResponse::Status204),
            Err(_) => Ok(UndoTransactionResponse::Status409),
        }
    }

    async fn redo_transaction(
        &self,
        _request: RedoTransactionRequest,
    ) -> Result<RedoTransactionResponse> {
        let mut transaction_manager = self.transaction_manager.lock().await;

        match transaction_manager.redo_transaction().await {
            Ok(_) => Ok(RedoTransactionResponse::Status204),
            Err(_) => Ok(RedoTransactionResponse::Status409),
        }
    }

    async fn get_messages(&self, _request: GetMessagesRequest) -> Result<GetMessagesResponse> {
        let receiver = self.editor_events_receiver.clone();

        let msg_future = async move {
            loop {
                match receiver.lock().await.recv().await {
                    Ok(editor_event) => {
                        let message = match editor_event {
                            EditorEvent::SelectionChanged(selections) => Message {
                                msg_type: MessageMsgType::SelectionChanged,
                                payload: serde_json::json!(selections).to_string(),
                            },
                            EditorEvent::ResourceChanged(changed_resources) => Message {
                                msg_type: MessageMsgType::ResourceChanged,
                                payload: serde_json::json!(changed_resources).to_string(),
                            },
                        };

                        return Ok(GetMessagesResponse::Status200(message));
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => {
                        return Err(Error::internal("message channel got closed"))
                    }
                }
            }
        };

        tokio::select! {
            response = msg_future => response,
            _ = sleep(Duration::from_secs(5)) => Ok(GetMessagesResponse::Status204)
        }
    }
}
