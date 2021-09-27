use crate::*;
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
            id: self.process_id.clone(),
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

    fn on_log_str(&mut self, level: LogLevel, msg: &'static str) {
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.push(LogMsgEvent { level, msg });
        if log_stream.is_full() {
            let old_event_block =
                log_stream.replace_block(Arc::new(LogMsgBlock::new(self.log_buffer_size)));
            assert!(!log_stream.is_full());
            self.sink
                .on_sink_event(TelemetrySinkEvent::OnLogBufferFull(old_event_block));
        }
    }

    fn on_thread_buffer_full(&mut self, stream: &mut ThreadStream) {
        let old_event_block =
            stream.replace_block(Arc::new(ThreadEventBlock::new(self.log_buffer_size)));
        assert!(!stream.is_full());
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnThreadBufferFull(old_event_block));
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
