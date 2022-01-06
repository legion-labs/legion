use std::collections::HashMap;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use log::LevelFilter;

use crate::event_block::TelemetryBlock;
use crate::metrics_block::MetricsStream;
use crate::{
    now, BeginScopeEvent, EndScopeEvent, EventSink, FloatMetricEvent, GetScopeDesc,
    IntegerMetricEvent, Level, LogBlock, LogDynMsgEvent, LogMsgEvent, LogStream, MetricDesc,
    MetricsBlock, NullEventSink, ProcessInfo, ThreadBlock, ThreadEventQueueTypeIndex, ThreadStream,
};

struct Dispatch {
    process_id: String,
    log_buffer_size: usize,
    thread_buffer_size: usize,
    metrics_buffer_size: usize,
    log_stream: Mutex<LogStream>,
    metrics_stream: Mutex<MetricsStream>,
    sink: Arc<dyn EventSink>,
}

impl Dispatch {
    pub fn new(
        log_buffer_size: usize,
        thread_buffer_size: usize,
        metrics_buffer_size: usize,
        sink: Arc<dyn EventSink>,
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
        obj.startup();
        obj.init_log_stream();
        obj.init_metrics_stream();
        obj
    }

    pub fn get_process_id(&self) -> String {
        self.process_id.clone()
    }

    fn shutdown(&mut self) {
        self.sink.on_shutdown();
        self.sink = Arc::new(NullEventSink {});
    }

    fn startup(&mut self) {
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
        self.sink.on_startup(process_info);
    }

    fn init_log_stream(&mut self) {
        let log_stream = self.log_stream.lock().unwrap();
        self.sink.on_init_log_stream(&log_stream);
    }

    fn init_metrics_stream(&mut self) {
        let metrics_stream = self.metrics_stream.lock().unwrap();
        self.sink.on_init_metrics_stream(&metrics_stream);
    }

    fn init_thread_stream(&mut self, cell: &Cell<Option<ThreadStream>>) {
        let mut properties = HashMap::new();
        properties.insert(String::from("thread-id"), thread_id::get().to_string());
        if let Some(name) = std::thread::current().name() {
            properties.insert("thread-name".to_owned(), name.to_owned());
        }
        let thread_stream = ThreadStream::new(
            self.thread_buffer_size,
            self.process_id.clone(),
            &["cpu".to_owned()],
            properties,
        );
        unsafe {
            let opt_ref = &mut *cell.as_ptr();
            self.sink.on_init_thread_stream(&thread_stream);
            *opt_ref = Some(thread_stream);
        }
    }

    #[inline]
    fn int_metric(&mut self, metric: &'static MetricDesc, value: u64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream.get_events_mut().push(IntegerMetricEvent {
            metric,
            value,
            time,
        });
        if metrics_stream.is_full() {
            // Release the lock before calling on_log_buffer_full
            drop(metrics_stream);
            self.flush_metrics_buffer();
        }
    }

    #[inline]
    fn float_metric(&mut self, metric: &'static MetricDesc, value: f64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream.get_events_mut().push(FloatMetricEvent {
            metric,
            value,
            time,
        });
        if metrics_stream.is_full() {
            drop(metrics_stream);
            self.flush_metrics_buffer();
        }
    }

    #[inline]
    fn flush_metrics_buffer(&mut self) {
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        if metrics_stream.is_empty() {
            return;
        }
        let stream_id = metrics_stream.stream_id().to_string();
        let mut old_event_block = metrics_stream.replace_block(Arc::new(MetricsBlock::new(
            self.metrics_buffer_size,
            stream_id,
        )));
        assert!(!metrics_stream.is_full());
        Arc::get_mut(&mut old_event_block).unwrap().close();
        self.sink.on_process_metrics_block(old_event_block);
    }

    #[inline]
    fn log_enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.sink.on_log_enabled(metadata)
    }

    fn log(&mut self, record: &log::Record<'_>) {
        self.sink.on_log(record);
        if let Some(static_str) = record.args().as_str() {
            self.log_static_str(record.level(), static_str);
        } else {
            self.log_string(
                record.level(),
                format!("target={} {}", record.metadata().target(), record.args()),
            );
        }
    }

    #[inline]
    fn log_static_str(&mut self, level: Level, msg: &'static str) {
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
            // Release the lock before calling on_log_buffer_full
            drop(log_stream);
            self.flush_log_buffer();
        }
    }

    #[inline]
    fn log_string(&mut self, level: Level, msg: String) {
        let time = now();
        let mut log_stream = self.log_stream.lock().unwrap();
        log_stream.get_events_mut().push(LogDynMsgEvent {
            time,
            level: level as u8,
            msg: lgn_transit::DynString(msg),
        });
        if log_stream.is_full() {
            // Release the lock before calling on_log_buffer_full()
            drop(log_stream);
            self.flush_log_buffer();
        }
    }

    #[inline]
    fn flush_log_buffer(&mut self) {
        let mut log_stream = self.log_stream.lock().unwrap();
        if log_stream.is_empty() {
            return;
        }
        let stream_id = log_stream.stream_id().to_string();
        let mut old_event_block =
            log_stream.replace_block(Arc::new(LogBlock::new(self.log_buffer_size, stream_id)));
        assert!(!log_stream.is_full());
        Arc::get_mut(&mut old_event_block).unwrap().close();
        self.sink.on_process_log_block(old_event_block);
    }

    #[inline]
    fn flush_thread_buffer(&mut self, stream: &mut ThreadStream) {
        if stream.is_empty() {
            return;
        }
        let mut old_block = stream.replace_block(Arc::new(ThreadBlock::new(
            self.thread_buffer_size,
            stream.stream_id().to_string(),
        )));
        assert!(!stream.is_full());
        Arc::get_mut(&mut old_block).unwrap().close();
        self.sink.on_process_thread_block(old_block);
    }
}

struct LogDispatch;

static mut G_DISPATCH: Option<Dispatch> = None;
static LOG_DISPATCHER: LogDispatch = LogDispatch;

thread_local! {
    static LOCAL_THREAD_STREAM: Cell<Option<ThreadStream>> = Cell::new(None);
}

pub fn init_event_dispatch(
    log_buffer_size: usize,
    thread_buffer_size: usize,
    metrics_buffer_size: usize,
    sink: Arc<dyn EventSink>,
) -> Result<(), String> {
    lazy_static::lazy_static! {
        static ref INIT_MUTEX: Mutex<()> = Mutex::new(());
    }
    let _guard = INIT_MUTEX.lock().unwrap();

    unsafe {
        if G_DISPATCH.is_none() {
            G_DISPATCH = Some(Dispatch::new(
                log_buffer_size,
                thread_buffer_size,
                metrics_buffer_size,
                sink,
            ));
            log::set_logger(&LOG_DISPATCHER).unwrap();
            Ok(())
        } else {
            log::info!("event dispatch already initialized");
            Err(String::from("event dispatch already initialized"))
        }
    }
}

impl log::Log for LogDispatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        unsafe {
            if let Some(d) = &mut G_DISPATCH {
                d.log_enabled(metadata)
            } else {
                false
            }
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        unsafe {
            if let Some(d) = &mut G_DISPATCH {
                d.log(record);
            }
        }
    }
    fn flush(&self) {}
}

#[inline]
pub fn set_max_log_level(level_filter: LevelFilter) {
    log::set_max_level(level_filter);
}

#[inline]
pub fn get_process_id() -> Option<String> {
    unsafe { G_DISPATCH.as_ref().map(Dispatch::get_process_id) }
}

#[inline]
pub fn shutdown_event_dispatch() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.shutdown();
        }
    }
}

#[inline]
pub fn record_int_metric(metric_desc: &'static MetricDesc, value: u64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.int_metric(metric_desc, value);
        }
    }
}

#[inline]
pub fn record_float_metric(metric_desc: &'static MetricDesc, value: f64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.float_metric(metric_desc, value);
        }
    }
}

#[inline]
pub fn flush_log_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.flush_log_buffer();
        }
    }
}

#[inline]
pub fn flush_metrics_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.flush_metrics_buffer();
        }
    }
}

//todo: should be implicit by default but limit the maximum number of tracked
// threads
#[inline]
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

#[inline]
pub fn flush_thread_buffer() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        let opt_stream = &mut *cell.as_ptr();
        if let Some(stream) = opt_stream {
            match &mut G_DISPATCH {
                Some(d) => {
                    d.flush_thread_buffer(stream);
                }
                None => {
                    panic!("threads are recording but there is no event dispatch");
                }
            }
        }
    });
}

#[inline]
fn on_thread_event<T>(event: T)
where
    T: lgn_transit::InProcSerialize + ThreadEventQueueTypeIndex,
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

#[inline]
pub fn on_begin_scope(scope: GetScopeDesc) {
    on_thread_event(BeginScopeEvent { time: now(), scope });
}

#[inline]
pub fn on_end_scope(scope: GetScopeDesc) {
    on_thread_event(EndScopeEvent { time: now(), scope });
}
