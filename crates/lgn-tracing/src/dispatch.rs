use std::collections::HashMap;
use std::fmt;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

use chrono::Utc;

pub use crate::errors::{Error, Result};
use crate::event::{EventSink, NullEventSink, TracingBlock};
use crate::logs::{
    LogBlock, LogMetadata, LogStaticStrEvent, LogStaticStrInteropEvent, LogStream, LogStringEvent,
    LogStringInteropEvent,
};
use crate::metrics::{
    FloatMetricEvent, IntegerMetricEvent, MetricMetadata, MetricsBlock, MetricsStream,
};
use crate::spans::{
    BeginThreadSpanEvent, EndThreadSpanEvent, ThreadBlock, ThreadEventQueueTypeIndex,
    ThreadSpanMetadata, ThreadStream,
};
use crate::{info, now, warn, Level, ProcessInfo};

pub fn init_event_dispatch(
    logs_buffer_size: usize,
    metrics_buffer_size: usize,
    threads_buffer_size: usize,
    sink: Arc<dyn EventSink>,
) -> Result<()> {
    lazy_static::lazy_static! {
        static ref INIT_MUTEX: Mutex<()> = Mutex::new(());
    }
    let _guard = INIT_MUTEX.lock().unwrap();

    unsafe {
        if G_DISPATCH.is_none() {
            G_DISPATCH = Some(Dispatch::new(
                logs_buffer_size,
                metrics_buffer_size,
                threads_buffer_size,
                sink,
            ));
            Ok(())
        } else {
            info!("event dispatch already initialized");
            Err(Error::AlreadyInitialized())
        }
    }
}

#[inline]
pub fn process_id() -> Option<String> {
    unsafe { G_DISPATCH.as_ref().map(Dispatch::get_process_id) }
}

pub fn shutdown_dispatch() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.shutdown();
        }
    }
}

#[inline(always)]
pub fn int_metric(metric_desc: &'static MetricMetadata, value: u64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.int_metric(metric_desc, value);
        }
    }
}

#[inline(always)]
pub fn float_metric(metric_desc: &'static MetricMetadata, value: f64) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.float_metric(metric_desc, value);
        }
    }
}

#[inline(always)]
pub fn log(desc: &'static LogMetadata, args: fmt::Arguments<'_>) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.log(desc, args);
        }
    }
}

#[inline(always)]
pub fn log_interop(desc: &LogMetadata, args: fmt::Arguments<'_>) {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.log_interop(desc, args);
        }
    }
}

#[inline(always)]
pub fn log_enabled(target: &str, level: Level) -> bool {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.log_enabled(level, target)
        } else {
            false
        }
    }
}

#[inline(always)]
pub fn flush_log_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.flush_log_buffer();
        }
    }
}

#[inline(always)]
pub fn flush_metrics_buffer() {
    unsafe {
        if let Some(d) = &mut G_DISPATCH {
            d.flush_metrics_buffer();
        }
    }
}

//todo: should be implicit by default but limit the maximum number of tracked
// threads
#[inline(always)]
pub fn init_thread_stream() {
    LOCAL_THREAD_STREAM.with(|cell| unsafe {
        if (*cell.as_ptr()).is_some() {
            return;
        }
        if let Some(d) = &mut G_DISPATCH {
            d.init_thread_stream(cell);
        } else {
            warn!("dispatch not initialized, cannot init thread stream, events will be lost for this thread");
        }
    });
}

#[inline(always)]
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

#[inline(always)]
pub fn on_begin_scope(scope: &'static ThreadSpanMetadata) {
    on_thread_event(BeginThreadSpanEvent {
        time: now(),
        thread_span_desc: scope,
    });
}

#[inline(always)]
pub fn on_end_scope(scope: &'static ThreadSpanMetadata) {
    on_thread_event(EndThreadSpanEvent {
        time: now(),
        thread_span_desc: scope,
    });
}

static mut G_DISPATCH: Option<Dispatch> = None;

thread_local! {
    static LOCAL_THREAD_STREAM: Cell<Option<ThreadStream>> = Cell::new(None);
}

#[inline(always)]
fn on_thread_event<T>(event: T)
where
    T: lgn_tracing_transit::InProcSerialize + ThreadEventQueueTypeIndex,
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

struct Dispatch {
    process_id: String,
    logs_buffer_size: usize,
    metrics_buffer_size: usize,
    threads_buffer_size: usize,
    log_stream: Mutex<LogStream>,
    metrics_stream: Mutex<MetricsStream>,
    sink: Arc<dyn EventSink>,
}

impl Dispatch {
    pub fn new(
        logs_buffer_size: usize,
        metrics_buffer_size: usize,
        threads_buffer_size: usize,
        sink: Arc<dyn EventSink>,
    ) -> Self {
        let process_id = uuid::Uuid::new_v4().to_string();
        let mut obj = Self {
            process_id: process_id.clone(),
            logs_buffer_size,
            metrics_buffer_size,
            threads_buffer_size,
            log_stream: Mutex::new(LogStream::new(
                logs_buffer_size,
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
            self.threads_buffer_size,
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
    fn int_metric(&mut self, desc: &'static MetricMetadata, value: u64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream
            .get_events_mut()
            .push(IntegerMetricEvent { desc, value, time });
        if metrics_stream.is_full() {
            // Release the lock before calling flush_metrics_buffer
            drop(metrics_stream);
            self.flush_metrics_buffer();
        }
    }

    #[inline]
    fn float_metric(&mut self, desc: &'static MetricMetadata, value: f64) {
        let time = now();
        let mut metrics_stream = self.metrics_stream.lock().unwrap();
        metrics_stream
            .get_events_mut()
            .push(FloatMetricEvent { desc, value, time });
        if metrics_stream.is_full() {
            drop(metrics_stream);
            // Release the lock before calling flush_metrics_buffer
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

    fn log_enabled(&mut self, level: Level, target: &str) -> bool {
        self.sink.on_log_enabled(level, target)
    }

    #[inline]
    fn log(&mut self, desc: &'static LogMetadata, args: fmt::Arguments<'_>) {
        let time = now();
        self.sink.on_log(desc, time, args);
        let mut log_stream = self.log_stream.lock().unwrap();
        if args.as_str().is_some() {
            log_stream
                .get_events_mut()
                .push(LogStaticStrEvent { desc, time });
        } else {
            log_stream.get_events_mut().push(LogStringEvent {
                desc,
                time,
                dyn_str: lgn_tracing_transit::DynString(args.to_string()),
            });
        }
        if log_stream.is_full() {
            // Release the lock before calling flush_log_buffer
            drop(log_stream);
            self.flush_log_buffer();
        }
    }

    #[inline]
    fn log_interop(&mut self, desc: &LogMetadata, args: fmt::Arguments<'_>) {
        let time = now();
        self.sink.on_log(desc, time, args);
        let mut log_stream = self.log_stream.lock().unwrap();
        if let Some(msg) = args.as_str() {
            log_stream.get_events_mut().push(LogStaticStrInteropEvent {
                time,
                level: desc.level as u32,
                target: desc.target.into(),
                msg: msg.into(),
            });
        } else {
            log_stream.get_events_mut().push(LogStringInteropEvent {
                time,
                level: desc.level as u32,
                target: desc.target.into(),
                msg: lgn_tracing_transit::DynString(args.to_string()),
            });
        }
        if log_stream.is_full() {
            // Release the lock before calling flush_log_buffer
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
            log_stream.replace_block(Arc::new(LogBlock::new(self.logs_buffer_size, stream_id)));
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
            self.threads_buffer_size,
            stream.stream_id().to_string(),
        )));
        assert!(!stream.is_full());
        Arc::get_mut(&mut old_block).unwrap().close();
        self.sink.on_process_thread_block(old_block);
    }
}
