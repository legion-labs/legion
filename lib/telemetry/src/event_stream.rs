use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::event_block::TelemetryBlock;

#[derive(Debug)]
pub struct EventStream<Block, DepsQueue> {
    stream_id: String,
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

    pub fn stream_id(&self) -> &str {
        self.stream_id.as_str()
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
