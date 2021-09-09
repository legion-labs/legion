use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct LogMsgEvent {
    pub level: LogLevel,
    pub msg: &'static str,
}

pub struct LogMsgBlock {
    pub events: Vec<LogMsgEvent>,
}

impl LogMsgBlock {
    pub fn new(buffer_size: usize) -> Self {
        let mut events = Vec::new();
        events.reserve(buffer_size);
        Self { events }
    }
}

pub struct LogStream {
    current_block: Arc<LogMsgBlock>,
    initial_size: usize,
}

impl LogStream {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            current_block: Arc::new(LogMsgBlock::new(buffer_size)),
            initial_size: buffer_size,
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
        self.current_block.events.len() + max_object_size > self.initial_size
    }

    fn get_events_mut(&mut self) -> &mut Vec<LogMsgEvent> {
        //get_mut_unchecked should be faster
        &mut Arc::get_mut(&mut self.current_block).unwrap().events
    }
}
