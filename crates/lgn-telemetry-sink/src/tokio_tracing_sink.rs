use std::fmt::Write;

use lgn_tracing::{dispatch::log_interop, logs::LogMetadata, Level};
use once_cell::sync::Lazy;
use tracing::{
    dispatcher::SetGlobalDefaultError,
    field::Field,
    span::{Attributes, Id, Record},
    subscriber, Event, Subscriber,
};
use tracing_subscriber::{
    layer::Context, prelude::*, registry::LookupSpan, EnvFilter, Layer, Registry,
};

// References:
// * https://docs.rs/tracing/latest/tracing/subscriber/index.html

#[derive(Default)]
pub(crate) struct TelemetryLayer {}

impl TelemetryLayer {
    pub(crate) fn setup() {
        static INIT_RESULT: Lazy<Result<(), SetGlobalDefaultError>> = Lazy::new(|| {
            let default_filter = format!("{}", ::tracing::Level::INFO);
            let filter_layer = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(&default_filter))
                .unwrap();
            let subscriber = Registry::default().with(filter_layer);

            let lgn_telemetry_layer = TelemetryLayer::default();
            let subscriber = subscriber.with(lgn_telemetry_layer);

            subscriber::set_global_default(subscriber)
        });
        assert!(INIT_RESULT.is_ok());
    }
}

impl<S> Layer<S> for TelemetryLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut recorder = Recorder::default();
        attrs.record(&mut recorder);

        if let Some(span_ref) = ctx.span(id) {
            span_ref.extensions_mut().insert(recorder);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span_ref) = ctx.span(id) {
            if let Some(recorder) = span_ref.extensions_mut().get_mut::<Recorder>() {
                values.record(recorder);
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut recorder = Recorder::default();
        event.record(&mut recorder);

        let meta = event.metadata();

        //let args = ;
        // if let Some(field) = event.fields().find(|field| field.name() == "message") {
        //     args = format_args!("{}", field);
        // }
        let log_desc = LogMetadata {
            level: tokio_tracing_level_to_level(*meta.level()),
            level_filter: std::sync::atomic::AtomicU32::new(0),
            fmt_str: "",
            target: meta.target(),
            module_path: meta.module_path().unwrap_or("unknown"),
            file: meta.file().unwrap_or("unknown"),
            line: meta.line().unwrap_or(0),
        };
        log_interop(&log_desc, format_args!("{:?}", event));
    }
}

#[derive(Default)]
struct Recorder {
    message: String,
    first_arg: bool,
}

impl tracing::field::Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            if !self.message.is_empty() {
                self.message = format!("{:?}\n{}", value, self.message);
            } else {
                self.message = format!("{:?}", value);
            }
        } else {
            if self.first_arg {
                // following args
                write!(self.message, " ").unwrap();
            } else {
                // first arg
                self.first_arg = true;
            }
            write!(self.message, "{} = {:?};", field.name(), value).unwrap();
        }
    }
}

impl std::fmt::Display for Recorder {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.message.is_empty() {
            write!(&mut f, "{}", self.message)
        } else {
            Ok(())
        }
    }
}

fn tokio_tracing_level_to_level(level: ::tracing::Level) -> Level {
    match level {
        ::tracing::Level::TRACE => Level::Trace,
        ::tracing::Level::DEBUG => Level::Debug,
        ::tracing::Level::INFO => Level::Info,
        ::tracing::Level::WARN => Level::Warn,
        ::tracing::Level::ERROR => Level::Error,
    }
}
