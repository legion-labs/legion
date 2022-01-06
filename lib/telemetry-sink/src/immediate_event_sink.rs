use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use lgn_telemetry::{
    log, log::Log, EventSink, LogBlock, LogStream, MetricsBlock, MetricsStream, ProcessInfo,
    ScopeEvent, ThreadBlock, ThreadEventQueueAny, ThreadStream,
};
use lgn_transit::HeterogeneousQueue;
use simple_logger::SimpleLogger;

pub struct ImmediateEventSink {
    simple_logger: SimpleLogger,
    chrome_trace_file: Option<String>,
    process_data: RwLock<Option<ProcessData>>,
    thread_names: RwLock<HashMap<String, String>>,
    chrome_events: Mutex<json::Array>,
}

struct ProcessData {
    tsc_frequency: u64,
    start_ticks: i64,
    process_id: String,
}

impl ImmediateEventSink {
    pub fn new(chrome_trace_file: Option<String>) -> Self {
        Self {
            simple_logger: SimpleLogger::new().with_utc_timestamps(),
            chrome_trace_file,
            process_data: RwLock::new(None),
            thread_names: RwLock::new(HashMap::new()),
            chrome_events: Mutex::new(json::Array::new()),
        }
    }
}

impl EventSink for ImmediateEventSink {
    fn on_startup(&self, proc_info: ProcessInfo) {
        let mut process_data = self.process_data.write().unwrap();
        *process_data = Some(ProcessData {
            tsc_frequency: proc_info.tsc_frequency,
            start_ticks: proc_info.start_ticks,
            process_id: proc_info.process_id,
        });
    }
    fn on_shutdown(&self) {
        if let Some(chrome_trace_file) = &self.chrome_trace_file {
            let chrome_events = self.chrome_events.lock().unwrap();
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
        let thread_name = thread_stream
            .properties()
            .get("thread-name")
            .or_else(|| thread_stream.properties().get("thread-id"));
        if let Some(thread_name) = thread_name {
            self.thread_names
                .write()
                .unwrap()
                .insert(thread_stream.stream_id().to_owned(), thread_name.clone());
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn on_process_thread_block(&self, block: Arc<ThreadBlock>) {
        if self.chrome_trace_file.is_none() {
            return;
        }
        let process_data = self.process_data.read().unwrap();
        if let Some(process_data) = &*process_data {
            if let Some(thread_name) = self.thread_names.read().unwrap().get(&block.stream_id) {
                for x in block.events.iter() {
                    let (phase, tick, name) = match x {
                        ThreadEventQueueAny::BeginScopeEvent(evt) => {
                            ("B", evt.time, evt.get_scope()().name)
                        }
                        ThreadEventQueueAny::EndScopeEvent(evt) => {
                            //record_scope_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                            ("E", evt.time, evt.get_scope()().name)
                        }
                    };
                    let time = 1000 * 1000 * (tick - process_data.start_ticks) as u64
                        / process_data.tsc_frequency;
                    let event = json::object! {
                        name: name,
                        cat: "PERF",
                        ph: phase,
                        pid: process_data.process_id.as_str(),
                        tid: thread_name.as_str(),
                        ts: time,

                    };
                    let mut events = self.chrome_events.lock().unwrap();
                    events.push(event);
                }
            }
        }
    }
}
