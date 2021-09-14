use crate::{LogMsgBlock, ThreadEventBlock};

pub trait EventBlockSink {
    fn on_log_buffer_full(&self, log_block: &LogMsgBlock);
    fn on_thread_buffer_full(&self, thread_block: &ThreadEventBlock);
}

pub struct NullEventSink {}
impl EventBlockSink for NullEventSink {
    fn on_log_buffer_full(&self, _log_block: &LogMsgBlock) {}
    fn on_thread_buffer_full(&self, _thread_block: &ThreadEventBlock) {}
}
