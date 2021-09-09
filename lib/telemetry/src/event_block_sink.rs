use crate::LogMsgBlock;

pub trait EventBlockSink {
    fn on_log_buffer_full(&self, log_block: &LogMsgBlock);
}
