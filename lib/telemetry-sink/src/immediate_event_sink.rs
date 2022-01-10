use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use lgn_tracing::{
    dispatch::{flush_log_buffer, log_enabled, log_interop},
    event::EventSink,
    logs::{LogBlock, LogMetadata, LogStream},
    max_level,
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadEventQueueAny, ThreadStream},
    Level, LevelFilter, ProcessInfo,
};
use lgn_tracing_transit::HeterogeneousQueue;
use simple_logger::SimpleLogger;

struct LogDispatch;

impl log::Log for LogDispatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let level = log_level_to_tracing_level(metadata.level());
        log_enabled(metadata.target(), level)
    }

    fn log(&self, record: &log::Record<'_>) {
        let level = log_level_to_tracing_level(record.level());
        let log_desc = LogMetadata {
            level,
            fmt_str: record.args().as_str().unwrap_or(""),
            target: record.module_path_static().unwrap_or("unknown"),
            module_path: record.module_path_static().unwrap_or("unknown"),
            file: record.file_static().unwrap_or("unknown"),
            line: record.line().unwrap_or(0),
        };
        log_interop(&log_desc, record.args());
    }
    fn flush(&self) {
        flush_log_buffer();
    }
}

pub struct ImmediateEventSink {
    simple_logger: SimpleLogger,
    chrome_trace_file: Option<String>,
    process_data: RwLock<Option<ProcessData>>,
    thread_ids: RwLock<HashMap<String, (u64, String)>>,
    chrome_events: Mutex<json::Array>,
}

struct ProcessData {
    tsc_frequency: u64,
    start_ticks: i64,
}

fn tracing_level_to_log_level(level: Level) -> log::Level {
    match level {
        Level::Error => log::Level::Error,
        Level::Warn => log::Level::Warn,
        Level::Info => log::Level::Info,
        Level::Debug => log::Level::Debug,
        Level::Trace => log::Level::Trace,
    }
}

fn log_level_to_tracing_level(level: log::Level) -> Level {
    match level {
        log::Level::Error => Level::Error,
        log::Level::Warn => Level::Warn,
        log::Level::Info => Level::Info,
        log::Level::Debug => Level::Debug,
        log::Level::Trace => Level::Trace,
    }
}

fn tracing_level_filter_to_log_level_filter(level: LevelFilter) -> log::LevelFilter {
    match level {
        LevelFilter::Off => log::LevelFilter::Off,
        LevelFilter::Error => log::LevelFilter::Error,
        LevelFilter::Warn => log::LevelFilter::Warn,
        LevelFilter::Info => log::LevelFilter::Info,
        LevelFilter::Debug => log::LevelFilter::Debug,
        LevelFilter::Trace => log::LevelFilter::Trace,
    }
}

impl ImmediateEventSink {
    pub fn new(chrome_trace_file: Option<String>) -> anyhow::Result<Self> {
        static LOG_DISPATCHER: LogDispatch = LogDispatch;
        log::set_logger(&LOG_DISPATCHER)
            .map_err(|_err| anyhow::anyhow!("Error creating immediate event sink"))?;

        log::set_max_level(tracing_level_filter_to_log_level_filter(max_level()));

        Ok(Self {
            simple_logger: SimpleLogger::new().with_utc_timestamps(),
            chrome_trace_file,
            process_data: RwLock::new(None),
            thread_ids: RwLock::new(HashMap::new()),
            chrome_events: Mutex::new(json::Array::new()),
        })
    }
}

impl EventSink for ImmediateEventSink {
    fn on_startup(&self, proc_info: ProcessInfo) {
        if self.chrome_trace_file.is_none() {
            return;
        }
        let mut process_data = self.process_data.write().unwrap();
        *process_data = Some(ProcessData {
            tsc_frequency: proc_info.tsc_frequency,
            start_ticks: proc_info.start_ticks,
        });
        let event = json::object! {
            ph: "M",
            name: "process_name",
            pid: 1,
            args: {
                name: proc_info.exe,
            },
        };
        let mut events = self.chrome_events.lock().unwrap();
        events.push(event);
    }
    fn on_shutdown(&self) {
        if let Some(chrome_trace_file) = &self.chrome_trace_file {
            let mut chrome_events = self.chrome_events.lock().unwrap();
            let thread_ids = self.thread_ids.read().unwrap();
            let mut id_names: Vec<_> = thread_ids.iter().map(|(_, id_name)| id_name).collect();
            id_names.sort_by(|(_, a), (_, b)| a.cmp(b));
            for (idx, (id, _)) in id_names.into_iter().enumerate() {
                let event = json::object! {
                    ph: "M",
                    name: "thread_sort_index",
                    pid: 1,
                    tid: *id,
                    args: {
                        sort_index: idx,
                    },
                };
                chrome_events.push(event);
            }
            let trace_document = json::object! {
                traceEvents: chrome_events.clone(),
            };
            if std::fs::write(chrome_trace_file, trace_document.dump()).is_ok() {
                println!("chrome trace written to {}", chrome_trace_file);
                println!("Open https://ui.perfetto.dev/ to view the trace");
            } else {
                println!("failed to write trace {}", chrome_trace_file);
            }
        }
    }

    fn on_log_enabled(&self, _level: Level, _target: &str) -> bool {
        true
    }

    fn on_log(&self, desc: &LogMetadata, _time: i64, args: &fmt::Arguments<'_>) {
        let lvl = tracing_level_to_log_level(desc.level);
        let record = log::RecordBuilder::new()
            .args(*args)
            .target(desc.target)
            .level(lvl)
            .file_static(Some(desc.file))
            .line(Some(desc.line))
            .module_path_static(Some(desc.module_path))
            .build();

        use log::Log;
        self.simple_logger.log(&record);
    }

    fn on_init_log_stream(&self, _: &LogStream) {}
    fn on_process_log_block(&self, _: Arc<LogBlock>) {}

    fn on_init_metrics_stream(&self, _: &MetricsStream) {}
    fn on_process_metrics_block(&self, _: Arc<MetricsBlock>) {}

    fn on_init_thread_stream(&self, thread_stream: &ThreadStream) {
        if self.chrome_trace_file.is_none() {
            return;
        }
        // Since perfetto doesn't support thread_sort_idx, we try at least to have the order
        // of threads as they come in. At least main thread is always first.
        static THREAD_ID_ASSIGNMENT: AtomicU64 = AtomicU64::new(0);
        let thread_id = thread_stream
            .properties()
            .get("thread-id")
            .expect("thread-id need to be set");
        let thread_name = thread_stream
            .properties()
            .get("thread-name")
            .or(Some(thread_id));
        let thread_id = THREAD_ID_ASSIGNMENT.fetch_add(1, Ordering::Relaxed);
        if let Some(thread_name) = thread_name {
            self.thread_ids.write().unwrap().insert(
                thread_stream.stream_id().to_owned(),
                (thread_id, thread_name.clone()),
            );
            let event = json::object! {
                ph: "M",
                name: "thread_name",
                pid: 1,
                tid: thread_id,
                args: {
                    name: thread_name.as_str(),
                },
            };
            let mut events = self.chrome_events.lock().unwrap();
            events.push(event);
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn on_process_thread_block(&self, block: Arc<ThreadBlock>) {
        if self.chrome_trace_file.is_none() {
            return;
        }
        {
            let events = self.chrome_events.lock().unwrap();
            if events.len() > 10_000_000 {
                return;
            }
        }
        let process_data = self.process_data.read().unwrap();
        if let Some(process_data) = &*process_data {
            if let Some((thread_id, _)) = self.thread_ids.read().unwrap().get(&block.stream_id) {
                for x in block.events.iter() {
                    let (phase, tick, name, file, line) = match x {
                        ThreadEventQueueAny::BeginThreadSpanEvent(evt) => (
                            "B",
                            evt.time,
                            evt.thread_span_desc.name,
                            evt.thread_span_desc.file,
                            evt.thread_span_desc.line,
                        ),
                        ThreadEventQueueAny::EndThreadSpanEvent(evt) => (
                            "E",
                            evt.time,
                            evt.thread_span_desc.name,
                            evt.thread_span_desc.file,
                            evt.thread_span_desc.line,
                        ),
                    };
                    let time = 1000.0 * 1000.0 * (tick - process_data.start_ticks) as f64
                        / process_data.tsc_frequency as f64;
                    let event = json::object! {
                        name: name,
                        cat: "PERF",
                        ph: phase,
                        pid: 1,
                        tid: *thread_id,
                        ts: time,
                        args: {
                            "[file]": file,
                            "[line]": line,
                        },
                    };
                    let mut events = self.chrome_events.lock().unwrap();
                    events.push(event);
                }
            }
        }
    }
}
