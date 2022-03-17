//! Telemetry Grpc sink library
//!
//! Provides logging, metrics, memory and performance profiling

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]

use std::any::TypeId;
use std::sync::{Arc, Mutex, Weak};
use std::{collections::HashMap, str::FromStr};

mod composite_event_sink;
mod grpc_event_sink;
mod local_event_sink;
mod stream;

use lgn_tracing::event::BoxedEventSink;
use lgn_tracing::info;
use lgn_tracing::{
    event::EventSink,
    guards::{TracingSystemGuard, TracingThreadGuard},
    LevelFilter,
};

pub type ProcessInfo = lgn_telemetry_proto::telemetry::Process;
pub type StreamInfo = lgn_telemetry_proto::telemetry::Stream;
pub type EncodedBlock = lgn_telemetry_proto::telemetry::Block;
pub use lgn_telemetry_proto::telemetry::ContainerMetadata;

use composite_event_sink::CompositeSink;
use grpc_event_sink::GRPCEventSink;
use local_event_sink::LocalEventSink;

pub struct TelemetryGuardBuilder {
    logs_buffer_size: usize,
    metrics_buffer_size: usize,
    threads_buffer_size: usize,
    target_max_levels: HashMap<String, String>,
    max_queue_size: isize,
    max_level_override: Option<LevelFilter>,
    interop_max_level_override: Option<LevelFilter>,
    local_sink_enabled: bool,
    local_sink_max_level: LevelFilter,
    grpc_sink_max_level: LevelFilter,
    extra_sinks: HashMap<TypeId, (LevelFilter, BoxedEventSink)>,
}

impl Default for TelemetryGuardBuilder {
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
            local_sink_enabled: lgn_config::config_get_or!("logging.local_sink_enabled", true),
            local_sink_max_level: LevelFilter::from_str(
                &(lgn_config::config_get!("logging.local_sink_max_level").unwrap_or_else(|| {
                    if cfg!(debug_assertions) {
                        "INFO".to_owned()
                    } else {
                        "WARN".to_owned()
                    }
                }) as String),
            )
            .unwrap_or(LevelFilter::Off),
            grpc_sink_max_level: LevelFilter::from_str(
                &(lgn_config::config_get!("logging.grpc_sink_max_level").unwrap_or_else(|| {
                    if cfg!(debug_assertions) {
                        "DEBUG".to_owned()
                    } else {
                        "INFO".to_owned()
                    }
                }) as String),
            )
            .unwrap_or(LevelFilter::Off),
            target_max_levels: lgn_config::config_get_or!("logging.level_filters", HashMap::new()),
            max_queue_size: 16, //todo: change to nb_threads * 2
            max_level_override: None,
            interop_max_level_override: None,
            extra_sinks: HashMap::default(),
        }
    }
}

impl TelemetryGuardBuilder {
    // Only one sink per type ?
    pub fn add_sink<Sink>(mut self, max_level: LevelFilter, sink: Sink) -> Self
    where
        Sink: EventSink + 'static,
    {
        let type_id = TypeId::of::<Sink>();

        self.extra_sinks
            .entry(type_id)
            .or_insert_with(|| (max_level, Box::new(sink)));

        self
    }

    /// Programmatic override
    pub fn with_max_level_override(mut self, level_filter: LevelFilter) -> Self {
        self.max_level_override = Some(level_filter);
        self
    }

    pub fn with_local_sink_enabled(mut self, enabled: bool) -> Self {
        self.local_sink_enabled = enabled;
        self
    }

    pub fn with_interop_max_level_override(mut self, level_filter: LevelFilter) -> Self {
        self.interop_max_level_override = Some(level_filter);
        self
    }

    pub fn with_local_sink_max_level(mut self, level_filter: LevelFilter) -> Self {
        self.local_sink_max_level = level_filter;
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

    pub fn build(self) -> anyhow::Result<TelemetryGuard> {
        let target_max_level: Vec<_> = self
            .target_max_levels
            .into_iter()
            .filter(|(key, _val)| key != "MAX_LEVEL")
            .map(|(key, val)| {
                (
                    key,
                    LevelFilter::from_str(val.as_str()).unwrap_or(LevelFilter::Off),
                )
            })
            .collect();

        let guard = {
            lazy_static::lazy_static! {
                static ref GLOBAL_WEAK_GUARD: Mutex<Weak<TracingSystemGuard>> = Mutex::new(Weak::new());
            }
            let mut weak_guard = GLOBAL_WEAK_GUARD.lock().unwrap();
            let weak = &mut *weak_guard;

            if let Some(arc) = weak.upgrade() {
                arc
            } else {
                let mut sinks: Vec<(LevelFilter, BoxedEventSink)> = vec![];
                if let Ok(url) = std::env::var("LEGION_TELEMETRY_URL") {
                    sinks.push((
                        self.grpc_sink_max_level,
                        Box::new(GRPCEventSink::new(&url, self.max_queue_size)),
                    ));
                }
                if self.local_sink_enabled {
                    sinks.push((self.local_sink_max_level, Box::new(LocalEventSink::new())));
                }
                let mut extra_sinks = self
                    .extra_sinks
                    .into_iter()
                    .map(|(_type_id, sink)| sink)
                    .collect();
                sinks.append(&mut extra_sinks);

                let sink: BoxedEventSink = Box::new(CompositeSink::new(
                    sinks,
                    target_max_level,
                    self.max_level_override,
                    self.interop_max_level_override,
                ));

                let arc = Arc::<TracingSystemGuard>::new(TracingSystemGuard::new(
                    self.logs_buffer_size,
                    self.metrics_buffer_size,
                    self.threads_buffer_size,
                    sink.into(),
                )?);
                *weak = Arc::<TracingSystemGuard>::downgrade(&arc);
                arc
            }
        };
        // order here is important
        Ok(TelemetryGuard {
            _guard: guard,
            _thread_guard: TracingThreadGuard::new(),
        })
    }
}

pub struct TelemetryGuard {
    // note we rely here on the drop order being the same as the declaration order
    _thread_guard: TracingThreadGuard,
    _guard: Arc<TracingSystemGuard>,
}

impl TelemetryGuard {
    pub fn default() -> anyhow::Result<Self> {
        TelemetryGuardBuilder::default().build()
    }
}
