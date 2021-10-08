use crate::{
    now, BeginScopeEvent, EndScopeEvent, EventBlockSink, GetScopeDesc, LogBlock, LogDynMsgEvent,
    LogLevel, LogMsgEvent, LogStream, NullEventSink, ProcessInfo, Stream, TelemetrySinkEvent,
    ThreadEventBlock, ThreadEventQueueTypeIndex, ThreadStream,
};
use chrono::Utc;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

struct Dispatch {
    process_id: String,
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
        let process_id = uuid::Uuid::new_v4().to_string();
        let mut obj = Self {
            process_id: process_id.clone(),
            log_buffer_size,
            thread_buffer_size,
            log_stream: Mutex::new(LogStream::new(log_buffer_size, process_id)),
            sink,
        };
        obj.on_init_process();
        obj.on_init_log_stream();
        obj
    }

    fn on_shutdown(&mut self) {
        self.sink.on_sink_event(TelemetrySinkEvent::OnShutdown);
        self.sink = Arc::new(NullEventSink {});
    }

    fn on_init_process(&mut self) {
        use raw_cpuid::CpuId;
        let start_time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, false);
        let cpuid = CpuId::new();
        let cpu_brand = cpuid
            .get_processor_brand_string()
            .map_or_else(|| "unknown".to_owned(), |b| b.as_str().to_owned());

        let tsc_frequency = match cpuid.get_tsc_info() {
            Some(tsc_info) => tsc_info.tsc_frequency().unwrap_or(0),
            None => 0,
        };

        let process_info = ProcessInfo {
            process_id: self.process_id.clone(),
            username: whoami::username(),
            realname: whoami::realname(),
            exe: std::env::current_exe()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            computer: whoami::devicename(),
            distro: whoami::distro(),
            cpu_brand,
            tsc_frequency,
            start_time,
        };
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnInitProcess(process_info));
    }

    fn on_init_log_stream(&mut self) {
        let log_stream = self.log_stream.lock().unwrap();
        self.sink.on_sink_event(TelemetrySinkEvent::OnInitStream(
            log_stream.get_stream_info(),
        ));
    }

    fn on_init_thread_stream(&mut self, stream: &ThreadStream) {
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnInitStream(stream.get_stream_info()));
    }

    fn on_log_str(&mut self, level: LogLevel, msg: &'static str) {
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.push(LogMsgEvent {
            level: level as u8,
            msg_len: msg.len() as u32,
            msg: msg.as_ptr(),
        });
        if log_stream.is_full() {
            drop(log_stream);
            self.on_log_buffer_full();
        }
    }

    fn on_log_string(&mut self, level: LogLevel, msg: String) {
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.push(LogDynMsgEvent {
            level: level as u8,
            msg: transit::DynString(msg),
        });
        if log_stream.is_full() {
            drop(log_stream);
            self.on_log_buffer_full();
        }
    }

    fn on_log_buffer_full(&mut self) {
        let mut log_stream = self.log_stream.lock().unwrap();
        let stream_id = log_stream.get_stream_id();
        let mut old_event_block =
            log_stream.replace_block(Arc::new(LogBlock::new(self.log_buffer_size, stream_id)));
        assert!(!log_stream.is_full());
        Arc::get_mut(&mut old_event_block).unwrap().close();
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnLogBufferFull(old_event_block));
    }

    fn on_thread_buffer_full(&mut self, stream: &mut ThreadStream) {
        let mut old_block = stream.replace_block(Arc::new(ThreadBlock::new(
            self.thread_buffer_size,
            stream.get_stream_id(),
        )));
        assert!(!stream.is_full());
        Arc::get_mut(&mut old_block).unwrap().close();
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnThreadBufferFull(old_block));
    }

    fn init_thread_stream(&mut self, cell: &Cell<Option<ThreadStream>>) {
        let thread_stream = ThreadStream::new(self.thread_buffer_size, self.process_id.clone());
        unsafe {
            let opt_ref = &mut *cell.as_ptr();
            self.on_init_thread_stream(&thread_stream);
            *opt_ref = Some(thread_stream);
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

pub fn shutdown_event_dispatch() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_shutdown();
        }
    }
}

pub fn log_str(level: LogLevel, msg: &'static str) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_log_str(level, msg);
        }
    }
}

pub fn log_string(level: LogLevel, msg: String) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_log_string(level, msg);
        }
    }
}

pub fn flush_log_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_log_buffer_full();
        }
    }
}

//todo: should be implicit by default but limit the maximum number of tracked threads
pub fn init_thread_stream() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.init_thread_stream(cell);
        } else {
            panic!("dispatch not initialized");
        }
    });
}

pub fn flush_thread_buffer() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        let opt_stream = &mut *cell.as_ptr();
        if let Some(stream) = opt_stream {
            match &mut G_DISPATCH {
                Some(d) => {
                    d.on_thread_buffer_full(stream);
                }
                None => {
                    panic!("threads are recording but there is no event dispatch");
                }
            }
        }
    });
}

fn on_thread_event<T>(event: T)
where
    T: transit::InProcSerialize + ThreadEventQueueTypeIndex,
{
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        let opt_stream = &mut *cell.as_ptr();
        if let Some(stream) = opt_stream {
            stream.push_event(event);
            if stream.is_full() {
                flush_thread_buffer();
            }
        }
    });
}

pub fn on_begin_scope(scope: GetScopeDesc) {
    on_thread_event(BeginScopeEvent { time: now(), scope });
}

pub fn on_end_scope(scope: GetScopeDesc) {
    on_thread_event(EndScopeEvent { time: now(), scope });
}
