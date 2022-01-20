use lgn_tracing::{dispatch::log_interop, logs::LogMetadata, Level};
use tracing::{span::Attributes, subscriber, Event, Id, Subscriber};
use tracing_subscriber::{layer::Context, prelude::*, EnvFilter, Layer, Registry};

#[derive(Default)]
pub(crate) struct TelemetryLayer {}

impl TelemetryLayer {
    pub(crate) fn setup() {
        let default_filter = format!("{}", ::tracing::Level::INFO);
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&default_filter))
            .unwrap();
        let subscriber = Registry::default().with(filter_layer);

        let lgn_telemetry_layer = Self::default();
        let subscriber = subscriber.with(lgn_telemetry_layer);

        subscriber::set_global_default(subscriber)
            .expect("Tokio default tracing subscriber already set");
    }
}

impl<S> Layer<S> for TelemetryLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        // panic!("event on_new_span");
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = match *meta.level() {
            ::tracing::Level::TRACE => Level::Trace,
            ::tracing::Level::DEBUG => Level::Debug,
            ::tracing::Level::INFO => Level::Info,
            ::tracing::Level::WARN => Level::Warn,
            ::tracing::Level::ERROR => Level::Error,
        };
        let args = format_args!("");
        // TODO extract fields
        // for field in event.fields() {
        //     field.name()
        // }
        let log_desc = LogMetadata {
            level,
            level_filter: std::sync::atomic::AtomicU32::new(0),
            fmt_str: "",
            target: meta.target(),
            module_path: meta.module_path().unwrap_or("unknown"),
            file: meta.file().unwrap_or("unknown"),
            line: meta.line().unwrap_or(0),
        };
        log_interop(&log_desc, &args);
    }
}
