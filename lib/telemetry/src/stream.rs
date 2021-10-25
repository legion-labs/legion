use crate::event_block::TelemetryBlock;
use crate::queue_metadata::make_queue_metedata;
use crate::{EncodedBlock, StreamInfo};
use anyhow::Result;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

pub trait Stream {
    fn get_stream_info(&self) -> StreamInfo;
    fn get_stream_id(&self) -> String;
}

pub trait StreamBlock {
    fn encode(&self) -> Result<EncodedBlock>;
}

pub struct EventStream<Block, DepsQueue> {
    pub stream_id: String,
    process_id: String,
    current_block: Arc<Block>,
    initial_size: usize,
    tags: Vec<String>,
    properties: HashMap<String, String>,
    _bogus: PhantomData<DepsQueue>,
}

impl<Block, DepsQueue> EventStream<Block, DepsQueue>
where
    Block: TelemetryBlock,
{
    pub fn new(
        buffer_size: usize,
        process_id: String,
        tags: &[String],
        properties: HashMap<String, String>,
    ) -> Self {
        let stream_id = uuid::Uuid::new_v4().to_string();
        Self {
            current_block: Arc::new(Block::new(buffer_size, stream_id.clone())),
            initial_size: buffer_size,
            stream_id,
            process_id,
            tags: tags.to_vec(),
            properties,
            _bogus: PhantomData::default(),
        }
    }

    pub fn replace_block(&mut self, new_block: Arc<Block>) -> Arc<Block> {
        let old_block = self.current_block.clone();
        self.current_block = new_block;
        old_block
    }

    pub fn is_full(&self) -> bool {
        let max_object_size = 1;
        self.current_block.len_bytes() + max_object_size > self.initial_size
    }

    pub fn is_empty(&self) -> bool {
        self.current_block.len_bytes() == 0
    }

    pub fn get_events_mut(&mut self) -> &mut Block::Queue {
        //get_mut_unchecked should be faster
        Arc::get_mut(&mut self.current_block).unwrap().events_mut()
    }

    pub fn get_process_id(&self) -> String {
        self.process_id.clone()
    }

    pub fn get_tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    pub fn get_properties(&self) -> HashMap<String, String> {
        self.properties.clone()
    }
}

impl<Block, DepsQueue> Stream for EventStream<Block, DepsQueue>
where
    Block: TelemetryBlock,
    DepsQueue: transit::HeterogeneousQueue,
    <Block as TelemetryBlock>::Queue: transit::HeterogeneousQueue,
{
    fn get_stream_info(&self) -> StreamInfo {
        let dependencies_meta = make_queue_metedata::<DepsQueue>();
        let obj_meta = make_queue_metedata::<Block::Queue>();
        StreamInfo {
            process_id: self.get_process_id(),
            stream_id: self.get_stream_id(),
            dependencies_metadata: Some(dependencies_meta),
            objects_metadata: Some(obj_meta),
            tags: self.get_tags(),
            properties: self.get_properties(),
        }
    }

    fn get_stream_id(&self) -> String {
        self.stream_id.clone()
    }
}
