use crate::*;
use std::sync::Arc;
use transit::*;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct LogMsgEvent {
    pub level: LogLevel,
    pub msg: &'static str,
}

impl Serialize for LogMsgEvent {}

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent> {}
);

#[derive(Debug)]
pub struct LogMsgBlock {
    pub events: LogMsgQueue,
}

impl LogMsgBlock {
    pub fn new(buffer_size: usize) -> Self {
        let events = LogMsgQueue::new(buffer_size);
        Self { events }
    }
}

pub struct LogStream {
    current_block: Arc<LogMsgBlock>,
    initial_size: usize,
    stream_id: String,
    process_id: String,
}

impl Stream for LogStream {
    fn get_stream_info(&self) -> StreamInfo {
        StreamInfo {
            process_id: self.process_id.clone(),
            stream_id: self.stream_id.clone(),
            dependencies_metadata: Some(telemetry_ingestion_proto::ContainerMetadata {
                types: vec![],
            }),
            objects_metadata: Some(telemetry_ingestion_proto::ContainerMetadata { types: vec![] }),
            tags: vec![String::from("log")],
        }
    }
}

impl LogStream {
    pub fn new(buffer_size: usize, process_id: String) -> Self {
        let stream_id = uuid::Uuid::new_v4().to_string();
        Self {
            current_block: Arc::new(LogMsgBlock::new(buffer_size)),
            initial_size: buffer_size,
            stream_id,
            process_id,
        }
    }

    pub fn replace_block(&mut self, new_block: Arc<LogMsgBlock>) -> Arc<LogMsgBlock> {
        let old_block = self.current_block.clone();
        self.current_block = new_block;
        old_block
    }

    pub fn push(&mut self, event: LogMsgEvent) {
        self.get_events_mut().push(event);
    }

    pub fn is_full(&self) -> bool {
        let max_object_size = 1;
        self.current_block.events.len_bytes() + max_object_size > self.initial_size
    }

    fn get_events_mut(&mut self) -> &mut LogMsgQueue {
        //get_mut_unchecked should be faster
        &mut Arc::get_mut(&mut self.current_block).unwrap().events
    }
}
