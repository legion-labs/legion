use crate::{LogMsgBlock, ProcessInfo, ThreadEventBlock};

pub trait EventBlockSink {
    fn on_init_process(&self, process_info: ProcessInfo);
    fn on_log_buffer_full(&self, log_block: &LogMsgBlock);
    fn on_thread_buffer_full(&self, thread_block: &ThreadEventBlock);
    fn on_shutdown(&self);
}

pub struct NullEventSink {}
impl EventBlockSink for NullEventSink {
    fn on_init_process(&self, _process_info: ProcessInfo) {}
    fn on_log_buffer_full(&self, _log_block: &LogMsgBlock) {}
    fn on_thread_buffer_full(&self, _thread_block: &ThreadEventBlock) {}
    fn on_shutdown(&self) {}
}
