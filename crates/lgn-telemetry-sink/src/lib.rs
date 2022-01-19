//! Telemetry Grpc sink library
//!
//! Provides logging, metrics, memory and performance profiling

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex, Weak};
use std::{collections::HashMap, str::FromStr};

mod grpc_event_sink;
mod immediate_event_sink;
mod stream;

use grpc_event_sink::GRPCEventSink;
use immediate_event_sink::ImmediateEventSink;

pub type ProcessInfo = lgn_telemetry_proto::telemetry::Process;
pub type StreamInfo = lgn_telemetry_proto::telemetry::Stream;
pub type EncodedBlock = lgn_telemetry_proto::telemetry::Block;
pub use lgn_telemetry_proto::telemetry::ContainerMetadata;
use lgn_tracing::event::NullEventSink;
use lgn_tracing::{
    event::EventSink,
    guards::{TracingSystemGuard, TracingThreadGuard},
    set_max_level, LevelFilter,
};
use lgn_tracing::{info, set_max_lod, LodFilter};

pub struct Config {
    logs_buffer_size: usize,
    metrics_buffer_size: usize,
    threads_buffer_size: usize,
    max_level: LevelFilter,
    level_filters: HashMap<String, String>,
    max_queue_size: isize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            logs_buffer_size: lgn_config::config_get_or!(
                "logging.logs_buffer_size",
                10 * 1024 * 1024
            ),
            metrics_buffer_size: lgn_config::config_get_or!(
                "logging.metrics_buffer_size",
                1024 * 1024
            ),
            threads_buffer_size: lgn_config::config_get_or!(
                "threads_buffer_size",
                10 * 1024 * 1024
            ),
            max_level: LevelFilter::from_str(
                &(lgn_config::config_get!("logging.max_level_filter").unwrap_or_else(|| {
                    if cfg!(debug_assertions) {
                        "INFO".to_owned()
                    } else {
                        "WARN".to_owned()
                    }
                }) as String),
            )
            .unwrap_or(LevelFilter::Off),
            level_filters: lgn_config::config_get_or!("logging.level_filters", HashMap::new()),
            max_queue_size: 16, //todo: change to nb_threads * 2
        }
    }
}

fn alloc_telemetry_system(
    config: Config,
    enable_console_printer: bool,
) -> anyhow::Result<Arc<TracingSystemGuard>> {
    lazy_static::lazy_static! {
        static ref GLOBAL_WEAK_GUARD: Mutex<Weak<TracingSystemGuard>> = Mutex::new(Weak::new());
    }
    let mut weak_guard = GLOBAL_WEAK_GUARD.lock().unwrap();
    let weak = &mut *weak_guard;
    if let Some(arc) = weak.upgrade() {
        return Ok(arc);
    }
    let sink: Arc<dyn EventSink> = match std::env::var("LEGION_TELEMETRY_URL") {
        Ok(url) => Arc::new(GRPCEventSink::new(&url, config.max_queue_size)),
        Err(_no_url_in_env) => {
            if enable_console_printer {
                Arc::new(ImmediateEventSink::new(
                    config.level_filters,
                    std::env::var("LGN_TRACE_FILE").ok(),
                )?)
            } else {
                Arc::new(NullEventSink {})
            }
        }
    };

    let arc = Arc::<TracingSystemGuard>::new(TracingSystemGuard::new(
        config.logs_buffer_size,
        config.metrics_buffer_size,
        config.threads_buffer_size,
        sink,
    )?);
    set_max_level(config.max_level);
    set_max_lod(LodFilter::Max);
    *weak = Arc::<TracingSystemGuard>::downgrade(&arc);
    Ok(arc)
}

pub struct TelemetryGuard {
    // note we rely here on the drop order being the same as the declaration order
    _thread_guard: TracingThreadGuard,
    _guard: Arc<TracingSystemGuard>,
}

impl TelemetryGuard {
    pub fn default() -> anyhow::Result<Self> {
        Self::new(Config::default(), true)
    }

    //todo: refac enable_console_printer, put in config?
    pub fn new(config: Config, enable_console_printer: bool) -> anyhow::Result<Self> {
        // order here is important
        Ok(Self {
            _guard: alloc_telemetry_system(config, enable_console_printer)?,
            _thread_guard: TracingThreadGuard::new(),
        })
    }
    pub fn with_log_level(self, level_filter: LevelFilter) -> Self {
        set_max_level(level_filter);
        log::set_max_level(
            immediate_event_sink::tracing_level_filter_to_log_level_filter(level_filter),
        );
        self
    }
    pub fn with_ctrlc_handling(self) -> Self {
        ctrlc::set_handler(move || {
            info!("Ctrl+C was hit!");
            lgn_tracing::guards::shutdown_telemetry();
            std::process::exit(1);
        })
        .expect("Error setting Ctrl+C handler");
        self
    }
}
