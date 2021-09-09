use crate::LogMsgBlock;

pub trait EventBlockSink {
    fn on_log_buffer_full(&self, log_block: &LogMsgBlock);
}

pub struct NullEventSink {}
impl EventBlockSink for NullEventSink {
    fn on_log_buffer_full(&self, _log_block: &LogMsgBlock) {}
}
