use crate::*;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

struct Dispatch {
    log_buffer_size: usize,
    thread_buffer_size: usize,
    log_stream: Mutex<LogStream>,
    sink: Arc<dyn EventBlockSink>,
}

impl Dispatch {
    pub fn new(
        log_buffer_size: usize,
        thread_buffer_size: usize,
        sink: Arc<dyn EventBlockSink>,
    ) -> Self {
        Self {
            log_buffer_size,
            thread_buffer_size,
            log_stream: Mutex::new(LogStream::new(log_buffer_size)),
            sink,
        }
    }

    fn on_log_str(&mut self, level: LogLevel, msg: &'static str) {
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.push(LogMsgEvent { level, msg });
        if log_stream.is_full() {
            let old_event_block =
                log_stream.replace_block(Arc::new(LogMsgBlock::new(self.log_buffer_size)));
            assert!(!log_stream.is_full());
            self.sink.on_log_buffer_full(&old_event_block);
        }
    }

    fn on_thread_buffer_full(&mut self, stream: &mut ThreadStream) {
        let old_event_block =
            stream.replace_block(Arc::new(ThreadEventBlock::new(self.log_buffer_size)));
        assert!(!stream.is_full());
        self.sink.on_thread_buffer_full(&old_event_block);
    }

    fn init_thread_stream(&mut self, cell: &Cell<Option<ThreadStream>>) {
        unsafe {
            let opt_ref = &mut *cell.as_ptr();
            *opt_ref = Some(ThreadStream::new(self.thread_buffer_size));
        }
    }
}

static mut G_DISPATCH: Option<Dispatch> = None;

thread_local! {
    static LOCAL_THREAD_STREAM: Cell<Option<ThreadStream>> = Cell::new(None);
}

pub fn init_event_dispatch(
    log_buffer_size: usize,
    thread_buffer_size: usize,
    sink: Arc<dyn EventBlockSink>,
) -> Result<(), String> {
    unsafe {
        if G_DISPATCH.is_some() {
            panic!("event dispatch already initialized");
        }
        G_DISPATCH = Some(Dispatch::new(log_buffer_size, thread_buffer_size, sink));
    }
    Ok(())
}

pub fn log_str(level: LogLevel, msg: &'static str) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_log_str(level, msg);
        }
    }
}

//todo: should be implicit by default but limit the maximum number of threads
pub fn init_thread_stream() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.init_thread_stream(cell);
        } else {
            panic!("dispatch not initialized");
        }
    });
}

pub fn on_begin_scope(scope: GetScopeDesc) {
    LOCAL_THREAD_STREAM.with(|cell| {
        unsafe {
            let opt_stream = &mut *cell.as_ptr();
            if let Some(stream) = opt_stream {
                stream.push_event(BeginScopeEvent {
                    time: now(),
                    get_scope_desc: scope,
                });
                //todo: refac
                if stream.is_full() {
                    match &mut G_DISPATCH {
                        Some(d) => {
                            d.on_thread_buffer_full(stream);
                        }
                        None => {
                            panic!("threads are recording but there is no event dispatch");
                        }
                    }
                }
            }
        }
    });
}

pub fn on_end_scope(scope: GetScopeDesc) {
    LOCAL_THREAD_STREAM.with(|cell| {
        unsafe {
            let opt_stream = &mut *cell.as_ptr();
            if let Some(stream) = opt_stream {
                stream.push_event(EndScopeEvent {
                    time: now(),
                    get_scope_desc: scope,
                });
                //todo: refac
                if stream.is_full() {
                    match &mut G_DISPATCH {
                        Some(d) => {
                            d.on_thread_buffer_full(stream);
                        }
                        None => {
                            panic!("threads are recording but there is no event dispatch");
                        }
                    }
                }
            }
        }
    });
}
