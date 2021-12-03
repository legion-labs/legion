use std::collections::HashMap;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

use chrono::Utc;

use crate::event_block::TelemetryBlock;
use crate::metrics_block::MetricsStream;
use crate::{
    now, BeginScopeEvent, EndScopeEvent, EventBlockSink, FloatMetricEvent, GetScopeDesc,
    IntegerMetricEvent, LogBlock, LogDynMsgEvent, LogLevel, LogMsgEvent, LogStream, MetricDesc,
    MetricsBlock, NullEventSink, ProcessInfo, Stream, TelemetrySinkEvent, ThreadBlock,
    ThreadEventQueueTypeIndex, ThreadStream,
};

struct Dispatch {
    process_id: String,
    log_buffer_size: usize,
    thread_buffer_size: usize,
    metrics_buffer_size: usize,
    log_stream: Mutex<LogStream>,
    metrics_stream: Mutex<MetricsStream>,
    sink: Arc<dyn EventBlockSink>,
}

impl Dispatch {
    pub fn new(
        log_buffer_size: usize,
        thread_buffer_size: usize,
        metrics_buffer_size: usize,
        sink: Arc<dyn EventBlockSink>,
    ) -> Self {
        let process_id = uuid::Uuid::new_v4().to_string();
        let mut obj = Self {
            process_id: process_id.clone(),
            log_buffer_size,
            thread_buffer_size,
            metrics_buffer_size,
            log_stream: Mutex::new(LogStream::new(
                log_buffer_size,
                process_id.clone(),
                &[String::from("log")],
                HashMap::new(),
            )),
            metrics_stream: Mutex::new(MetricsStream::new(
                metrics_buffer_size,
                process_id,
                &[String::from("metrics")],
                HashMap::new(),
            )),
            sink,
        };
        obj.on_init_process();
        obj.on_init_log_stream();
        obj.on_init_metrics_stream();
        obj
    }

    pub fn get_process_id(&self) -> String {
        self.process_id.clone()
    }

    fn on_shutdown(&mut self) {
        self.sink.on_sink_event(TelemetrySinkEvent::OnShutdown);
        self.sink = Arc::new(NullEventSink {});
    }

    fn on_init_process(&mut self) {
        use raw_cpuid::CpuId;

        let mut parent_process = String::new();
        if let Ok(parent_process_guid) = std::env::var("LEGION_TELEMETRY_PARENT_PROCESS") {
            parent_process = parent_process_guid;
        }
        std::env::set_var("LEGION_TELEMETRY_PARENT_PROCESS", &self.process_id);

        let start_ticks = now();
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
            start_ticks,
            parent_process_id: parent_process,
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

    fn on_init_metrics_stream(&mut self) {
        let metrics_stream = self.metrics_stream.lock().unwrap();
        self.sink.on_sink_event(TelemetrySinkEvent::OnInitStream(
            metrics_stream.get_stream_info(),
        ));
    }

    fn on_init_thread_stream(&mut self, stream: &ThreadStream) {
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnInitStream(stream.get_stream_info()));
    }

    fn on_int_metric(&mut self, metric: &'static MetricDesc, value: u64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream.get_events_mut().push(IntegerMetricEvent {
            metric,
            value,
            time,
        });
        if metrics_stream.is_full() {
            drop(metrics_stream);
            self.on_metrics_buffer_full();
        }
    }

    fn on_float_metric(&mut self, metric: &'static MetricDesc, value: f64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream.get_events_mut().push(FloatMetricEvent {
            metric,
            value,
            time,
        });
        if metrics_stream.is_full() {
            drop(metrics_stream);
            self.on_metrics_buffer_full();
        }
    }

    fn on_metrics_buffer_full(&mut self) {
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        if metrics_stream.is_empty() {
            return;
        }
        let stream_id = metrics_stream.get_stream_id();
        let mut old_event_block = metrics_stream.replace_block(Arc::new(MetricsBlock::new(
            self.metrics_buffer_size,
            stream_id,
        )));
        assert!(!metrics_stream.is_full());
        Arc::get_mut(&mut old_event_block).unwrap().close();
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnMetricsBufferFull(old_event_block));
    }

    fn on_log_str(&mut self, level: LogLevel, msg: &'static str) {
        let time = now();
        let mut log_stream = self.log_stream.lock().unwrap();
        if log_stream.is_empty() {
            return;
        }
        log_stream.get_events_mut().push(LogMsgEvent {
            time,
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
        let time = now();
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.get_events_mut().push(LogDynMsgEvent {
            time,
            level: level as u8,
            msg: legion_transit::DynString(msg),
        });
        if log_stream.is_full() {
            drop(log_stream);
            self.on_log_buffer_full();
        }
    }

    fn on_log_buffer_full(&mut self) {
        let mut log_stream = self.log_stream.lock().unwrap();
        if log_stream.is_empty() {
            return;
        }
        let stream_id = log_stream.get_stream_id();
        let mut old_event_block =
            log_stream.replace_block(Arc::new(LogBlock::new(self.log_buffer_size, stream_id)));
        assert!(!log_stream.is_full());
        Arc::get_mut(&mut old_event_block).unwrap().close();
        self.sink
            .on_sink_event(TelemetrySinkEvent::OnLogBufferFull(old_event_block));
    }

    fn on_thread_buffer_full(&mut self, stream: &mut ThreadStream) {
        if stream.is_empty() {
            return;
        }
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
        let mut properties = HashMap::new();
        properties.insert(String::from("thread-id"), format!("{}", thread_id::get()));
        let thread_stream = ThreadStream::new(
            self.thread_buffer_size,
            self.process_id.clone(),
            &[String::from("cpu")],
            properties,
        );
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
    metrics_buffer_size: usize,
    make_sink: &mut dyn FnMut() -> Arc<dyn EventBlockSink>,
) -> Result<(), String> {
    lazy_static::lazy_static! {
        static ref INIT_MUTEX: Mutex<()> = Mutex::new(());
    }
    let _guard = INIT_MUTEX.lock().unwrap();

    unsafe {
        if G_DISPATCH.is_none() {
            let sink: Arc<dyn EventBlockSink> = make_sink();
            G_DISPATCH = Some(Dispatch::new(
                log_buffer_size,
                thread_buffer_size,
                metrics_buffer_size,
                sink,
            ));
            Ok(())
        } else {
            log::info!("event dispatch already initialized");
            Err(String::from("event dispatch already initialized"))
        }
    }
}

pub fn get_process_id() -> Option<String> {
    unsafe { G_DISPATCH.as_ref().map(Dispatch::get_process_id) }
}

pub fn shutdown_event_dispatch() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_shutdown();
        }
    }
}

pub fn record_int_metric(metric_desc: &'static MetricDesc, value: u64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_int_metric(metric_desc, value);
        }
    }
}

pub fn record_float_metric(metric_desc: &'static MetricDesc, value: f64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_float_metric(metric_desc, value);
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

pub fn flush_metrics_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.on_metrics_buffer_full();
        }
    }
}

//todo: should be implicit by default but limit the maximum number of tracked threads
pub fn init_thread_stream() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        if (*cell.as_ptr()).is_some() {
            return;
        }
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
    T: legion_transit::InProcSerialize + ThreadEventQueueTypeIndex,
{
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        let opt_stream = &mut *cell.as_ptr();
        if let Some(stream) = opt_stream {
            stream.get_events_mut().push(event);
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
