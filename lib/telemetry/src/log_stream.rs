use crate::*;
use std::sync::Arc;
use transit::*;

pub struct LogStream {
    current_block: Arc<LogBlock>,
    initial_size: usize,
    stream_id: String,
    process_id: String,
}

impl Stream for LogStream {
    fn get_stream_info(&self) -> StreamInfo {
        let dependencies_udts = <LogMsgQueue as ReflectiveQueue>::reflect_contained()
            .iter()
            .map(|udt| telemetry_ingestion_proto::UserDefinedType {
                name: udt.name.to_owned(),
                size: udt.size as u32,
                members: udt
                    .members
                    .iter()
                    .map(|member| telemetry_ingestion_proto::UdtMember {
                        name: member.name.to_owned(),
                        type_name: member.type_name.to_owned(),
                        offset: member.offset as u32,
                        size: member.size as u32,
                        is_reference: member.is_reference,
                    })
                    .collect(),
            })
            .collect();
        StreamInfo {
            process_id: self.process_id.clone(),
            stream_id: self.stream_id.clone(),
            dependencies_metadata: Some(telemetry_ingestion_proto::ContainerMetadata {
                types: dependencies_udts,
            }),
            objects_metadata: Some(telemetry_ingestion_proto::ContainerMetadata { types: vec![] }),
            tags: vec![String::from("log")],
        }
    }

    fn get_stream_id(&self) -> String {
        self.stream_id.clone()
    }
}

impl LogStream {
    pub fn new(buffer_size: usize, process_id: String) -> Self {
        let stream_id = uuid::Uuid::new_v4().to_string();
        Self {
            current_block: Arc::new(LogBlock::new(buffer_size, stream_id.clone())),
            initial_size: buffer_size,
            stream_id,
            process_id,
        }
    }

    pub fn replace_block(&mut self, new_block: Arc<LogBlock>) -> Arc<LogBlock> {
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
