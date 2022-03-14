//! Telemetry Grpc sink library
//!
//! Provides logging, metrics, memory and performance profiling

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]

use std::any::TypeId;
use std::sync::{Arc, Mutex, Weak};
use std::{collections::HashMap, str::FromStr};

mod grpc_event_sink;
mod immediate_event_sink;
mod stream;
#[cfg(feature = "tokio-tracing")]
mod tokio_tracing_sink;

use grpc_event_sink::GRPCEventSink;
use immediate_event_sink::ImmediateEventSink;

pub type ProcessInfo = lgn_telemetry_proto::telemetry::Process;
pub type StreamInfo = lgn_telemetry_proto::telemetry::Stream;
pub type EncodedBlock = lgn_telemetry_proto::telemetry::Block;
pub use lgn_telemetry_proto::telemetry::ContainerMetadata;
use lgn_tracing::event::{BoxedEventSink, NullEventSink};
use lgn_tracing::{
    event::EventSink,
    guards::{TracingSystemGuard, TracingThreadGuard},
    set_max_level, LevelFilter,
};
use lgn_tracing::{info, set_max_lod, LodFilter};

pub struct Config {
    pub logs_buffer_size: usize,
    pub metrics_buffer_size: usize,
    pub threads_buffer_size: usize,
    pub max_level: LevelFilter,
    level_filters: HashMap<String, String>,
    pub max_queue_size: isize,
    pub enable_console_printer: bool,
    pub enable_tokio_console_server: bool,
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
            enable_console_printer: true,
            enable_tokio_console_server: false,
        }
    }
}

fn alloc_telemetry_system(
    config: Config,
    custom_sinks: &mut Option<Vec<BoxedEventSink>>,
) -> anyhow::Result<Arc<TracingSystemGuard>> {
    lazy_static::lazy_static! {
        static ref GLOBAL_WEAK_GUARD: Mutex<Weak<TracingSystemGuard>> = Mutex::new(Weak::new());
    }
    let mut weak_guard = GLOBAL_WEAK_GUARD.lock().unwrap();
    let weak = &mut *weak_guard;
    if let Some(arc) = weak.upgrade() {
        return Ok(arc);
    }
    let sinks: BoxedEventSink = match std::env::var("LEGION_TELEMETRY_URL") {
        Ok(url) => Box::new(GRPCEventSink::new(&url, config.max_queue_size)),
        Err(_no_url_in_env) => {
            if config.enable_console_printer {
                Box::new(ImmediateEventSink::new(
                    config.level_filters,
                    std::env::var("LGN_TRACE_FILE").ok(),
                )?)
            } else {
                Box::new(NullEventSink {})
            }
        }
    };

    let mut sinks = vec![sinks];

    if let Some(custom_sinks) = custom_sinks {
        sinks.append(custom_sinks);
    }

    let sinks = Arc::new(sinks);

    let arc = Arc::<TracingSystemGuard>::new(TracingSystemGuard::new(
        config.logs_buffer_size,
        config.metrics_buffer_size,
        config.threads_buffer_size,
        sinks,
    )?);
    set_max_level(config.max_level);
    set_max_lod(LodFilter::Max);
    *weak = Arc::<TracingSystemGuard>::downgrade(&arc);
    Ok(arc)
}

#[derive(Default)]
pub struct TelemetryGuardBuilder {
    config: Config,
    level_filter: Option<LevelFilter>,
    ctrcl_handling: bool,
    sinks: HashMap<TypeId, BoxedEventSink>,
}

impl TelemetryGuardBuilder {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    // Only one sink per type ?
    pub fn add_sink<Sink>(mut self, sink: Sink) -> Self
    where
        Sink: EventSink + 'static,
    {
        let type_id = TypeId::of::<Sink>();

        self.sinks.entry(type_id).or_insert_with(|| Box::new(sink));

        self
    }

    pub fn log_level(mut self, level_filter: LevelFilter) -> Self {
        self.level_filter = Some(level_filter);

        self
    }

    pub fn with_ctrlc_handling(mut self) -> Self {
        self.ctrcl_handling = true;

        self
    }

    pub fn build(self) -> anyhow::Result<TelemetryGuard> {
        #[cfg(feature = "tokio-tracing")]
        tokio_tracing_sink::TelemetryLayer::setup(self.config.enable_tokio_console_server);

        // order here is important
        let mut telemetry_guard = TelemetryGuard {
            _guard: alloc_telemetry_system(
                self.config,
                &mut Some(
                    self.sinks
                        .into_iter()
                        .map(|(_type_id, sink)| sink)
                        .collect(),
                ),
            )?,
            _thread_guard: TracingThreadGuard::new(),
        };

        if self.ctrcl_handling {
            telemetry_guard = telemetry_guard.with_ctrlc_handling();
        }

        if let Some(level_filter) = self.level_filter {
            telemetry_guard = telemetry_guard.with_log_level(level_filter);
        }

        Ok(telemetry_guard)
    }
}

pub struct TelemetryGuard {
    // note we rely here on the drop order being the same as the declaration order
    _thread_guard: TracingThreadGuard,
    _guard: Arc<TracingSystemGuard>,
}

impl TelemetryGuard {
    pub fn default() -> anyhow::Result<Self> {
        Self::new(Config::default())
    }

    //todo: refac enable_console_printer, put in config?
    pub fn new(config: Config) -> anyhow::Result<Self> {
        #[cfg(feature = "tokio-tracing")]
        tokio_tracing_sink::TelemetryLayer::setup(config.enable_tokio_console_server);

        // order here is important
        Ok(Self {
            _guard: alloc_telemetry_system(config, &mut None)?,
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
