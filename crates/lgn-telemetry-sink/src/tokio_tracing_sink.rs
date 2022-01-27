use std::fmt::Write;

use console_subscriber::ConsoleLayer;
use lgn_tracing::{dispatch::log_interop, logs::LogMetadata, Level};
use once_cell::sync::Lazy;
use tracing::{
    dispatcher::SetGlobalDefaultError,
    field::Field,
    span::{Attributes, Id, Record},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, prelude::*, registry::LookupSpan, EnvFilter, Layer};

// References:
// * https://docs.rs/tracing/latest/tracing/subscriber/index.html

#[derive(Default)]
pub(crate) struct TelemetryLayer {}

impl TelemetryLayer {
    pub(crate) fn setup() {
        static INIT_RESULT: Lazy<Result<(), SetGlobalDefaultError>> = Lazy::new(|| {
            // get default filters
            let env_filter_layer = EnvFilter::try_from_default_env()
                .or_else(|_| {
                    EnvFilter::try_new(format!(
                        "{},tokio=trace,runtime=trace",
                        ::tracing::Level::INFO
                    ))
                })
                .unwrap();

            // spawn the console server in the background, returning a `Layer`
            let console_layer = ConsoleLayer::builder()
                .with_default_env()
                //.server_addr(([127, 0, 0, 1], 61234))
                .spawn();

            // redirect tokio tracing events and spans to telemetry
            let lgn_telemetry_layer = TelemetryLayer::default();

            let subscriber = tracing_subscriber::registry()
                .with(env_filter_layer)
                .with(console_layer)
                .with(lgn_telemetry_layer);

            tracing::subscriber::set_global_default(subscriber)
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
        let meta = event.metadata();
        let log_desc = LogMetadata {
            level: tokio_tracing_level_to_level(*meta.level()),
            level_filter: std::sync::atomic::AtomicU32::new(0),
            fmt_str: "", // cannot extract static format str from field visitor :(
            target: meta.target(),
            module_path: meta.module_path().unwrap_or("unknown"),
            file: meta.file().unwrap_or("unknown"),
            line: meta.line().unwrap_or(0),
        };

        let mut recorder = Recorder::default();
        event.record(&mut recorder);
        log_interop(&log_desc, format_args!("{}", recorder.message));
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
