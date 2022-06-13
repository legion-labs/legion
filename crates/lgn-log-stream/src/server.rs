use async_trait::async_trait;

use crate::api::log::{
    requests::LogEntriesRequest, responses::LogEntriesResponse, Api, TraceEvent, TraceEventLevel,
};
use lgn_async::receiver::SharedUnboundedReceiver;
use lgn_online::server::Result;

pub type TraceEventsReceiver = SharedUnboundedReceiver<crate::TraceEvent>;

/// The `api` implementation for the log server.
pub(crate) struct Server {
    #[allow(dead_code)]
    trace_events_receiver: TraceEventsReceiver,
}

impl Server {
    pub(crate) fn new(trace_events_receiver: TraceEventsReceiver) -> Self {
        Self {
            trace_events_receiver,
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn log_entries(
        &self,
        _parts: http::request::Parts,
        _request: LogEntriesRequest,
    ) -> Result<LogEntriesResponse> {
        Ok(LogEntriesResponse::Status200(vec![TraceEvent {
            level: TraceEventLevel::Debug,
            message: "It works".into(),
            target: "log::server".into(),
            time: 42,
        }]))
        // let (tx, rx) = mpsc::unbounded_channel();
        // let receiver = self.trace_events_receiver.clone();

        // tokio::spawn(async move {
        //     loop {
        //         match receiver.lock().await.recv().await {
        //             Ok(TraceEvent::Message {
        //                 target,
        //                 message,
        //                 level,
        //                 time,
        //             }) => {
        //                 if let Err(_error) = tx.send(Ok(InitLogStreamResponse {
        //                     response: Some(init_log_stream_response::Response::TraceEvent(
        //                         lgn_log_stream_proto::log_stream::TraceEvent {
        //                             // There must be a default, zero, value for enums but Level is 1-indexed
        //                             // (https://developers.google.com/protocol-buffers/docs/proto3#enum)
        //                             // So we simply decrement the level to get the proper value at runtime
        //                             level: (level as i32 - 1),
        //                             message,
        //                             target,
        //                             time,
        //                         },
        //                     )),
        //                 })) {
        //                     // Sent errors are always related to closed connection:
        //                     // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
        //                     // So we can stop the task
        //                     return;
        //                 }
        //             }
        //             Ok(_trace_event) => {
        //                 // Ignoring other events for now
        //             }
        //             Err(RecvError::Lagged(skipped_messages)) => {
        //                 if let Err(_error) = tx.send(Ok(InitLogStreamResponse {
        //                     response: Some(init_log_stream_response::Response::Lagging(
        //                         skipped_messages,
        //                     )),
        //                 })) {
        //                     // Sent errors are always related to closed connection:
        //                     // https://github.com/tokio-rs/tokio/blob/b1afd95994be0d46ea70ba784439a684a787f50e/tokio/src/sync/mpsc/error.rs#L12
        //                     // So we can stop the task
        //                     return;
        //                 }
        //             }
        //             Err(RecvError::Closed) => return,
        //         }
        //     }
        // });

        // Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
