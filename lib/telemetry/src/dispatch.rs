use crate::*;
use std::sync::Arc;

struct Dispatch {
    log_buffer_size: usize,
    log_stream: LogStream,
    sink: Arc<dyn EventBlockSink>,
}

impl Dispatch {
    pub fn new(log_buffer_size: usize, sink: Arc<dyn EventBlockSink>) -> Self {
        Self {
            log_buffer_size,
            log_stream: LogStream::new(log_buffer_size),
            sink,
        }
    }

    fn log_str(&mut self, level: LogLevel, msg: &'static str) {
        self.log_stream.push(LogMsgEvent { level, msg });
        if self.log_stream.is_full() {
            self.sink.on_log_buffer_full(&mut self.log_stream);
            self.log_stream = LogStream::new(self.log_buffer_size);
            assert!(!self.log_stream.is_full());
        }
    }
}

static mut G_DISPATCH: Option<Dispatch> = None;

pub fn init_event_dispatch(
    log_buffer_size: usize,
    sink: Arc<dyn EventBlockSink>,
) -> Result<(), String> {
    unsafe {
        if G_DISPATCH.is_some() {
            panic!("event dispatch already initialized");
        }
        G_DISPATCH = Some(Dispatch::new(log_buffer_size, sink));
    }
    Ok(())
}

pub fn log_str(level: LogLevel, msg: &'static str) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.log_str(level, msg);
        }
    }
}
