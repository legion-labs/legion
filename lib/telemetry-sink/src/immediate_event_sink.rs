use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use lgn_tracing::{
    log, log::Log, EventSink, LogBlock, LogStream, MetricsBlock, MetricsStream, ProcessInfo,
    ThreadBlock, ThreadEventQueueAny, ThreadStream,
};
use lgn_transit::HeterogeneousQueue;
use simple_logger::SimpleLogger;

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

impl ImmediateEventSink {
    pub fn new(chrome_trace_file: Option<String>) -> Self {
        Self {
            simple_logger: SimpleLogger::new().with_utc_timestamps(),
            chrome_trace_file,
            process_data: RwLock::new(None),
            thread_ids: RwLock::new(HashMap::new()),
            chrome_events: Mutex::new(json::Array::new()),
        }
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

    fn on_log_enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.simple_logger.enabled(metadata)
    }
    fn on_log(&self, record: &log::Record<'_>) {
        self.simple_logger.log(record);
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
                        ThreadEventQueueAny::BeginScopeEvent(evt) => (
                            "B",
                            evt.time,
                            evt.scope.name,
                            evt.scope.filename,
                            evt.scope.line,
                        ),
                        ThreadEventQueueAny::EndScopeEvent(evt) => (
                            "E",
                            evt.time,
                            evt.scope.name,
                            evt.scope.filename,
                            evt.scope.line,
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
