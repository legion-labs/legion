use std::{sync::Arc};

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
    pub current_block: Arc<LogMsgBlock>,
}

impl LogStream {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            current_block: Arc::new(LogMsgBlock::new(buffer_size)),
        }
    }

    pub fn get_events( &self ) -> &Vec<LogMsgEvent>{
        //get_mut_unchecked should be faster
        &self.current_block.events
    }
    
    pub fn push(&mut self, event: LogMsgEvent) {
        self.get_events_mut().push(event);
    }

    pub fn clear(&mut self) {
        self.get_events_mut().clear();
    }

    pub fn is_full(&self) -> bool {
        self.current_block.events.capacity() == self.current_block.events.len()
    }

    fn get_events_mut( &mut self ) -> &mut Vec<LogMsgEvent>{
        //get_mut_unchecked should be faster
        &mut Arc::get_mut(&mut self.current_block).unwrap().events
    }
    
}
