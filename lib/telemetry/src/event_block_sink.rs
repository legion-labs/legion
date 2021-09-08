use crate::LogStream;


pub trait EventBlockSink {
    fn on_log_buffer_full(&self, log_stream: &mut LogStream);
}
