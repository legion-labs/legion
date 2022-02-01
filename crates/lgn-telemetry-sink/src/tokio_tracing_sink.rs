use std::fmt::Write;

use console_subscriber::ConsoleLayer;
use lgn_tracing::{dispatch::log_interop, logs::LogMetadata, Level};
use once_cell::sync::OnceCell;
use tracing::{
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
    pub(crate) fn setup(enable_tokio_console_server: bool) {
        static TRACING_SUBSCRIBER_INIT: OnceCell<()> = OnceCell::new();

        TRACING_SUBSCRIBER_INIT.get_or_init(|| {
            let subscriber = tracing_subscriber::registry();

            // get default filters
            let subscriber = {
                let env_filter_layer = EnvFilter::try_from_default_env()
                    .or_else(|_| {
                        EnvFilter::try_new(format!(
                            "{},tokio=trace,runtime=trace",
                            ::tracing::Level::INFO
                        ))
                    })
                    .unwrap();

                subscriber.with(env_filter_layer)
            };

            // redirect tokio tracing events and spans to telemetry
            let subscriber = {
                let lgn_telemetry_layer = Self::default();

                subscriber.with(lgn_telemetry_layer)
            };

            if enable_tokio_console_server {
                // spawn the console server in the background, returning a `Layer`
                let console_layer = ConsoleLayer::builder()
                    .with_default_env()
                    //.server_addr(([127, 0, 0, 1], 61234))
                    .spawn();

                let subscriber = subscriber.with(console_layer);

                tracing::subscriber::set_global_default(subscriber)
                    .expect("unable to set global tracing subscriber");
            } else {
                tracing::subscriber::set_global_default(subscriber)
                    .expect("unable to set global tracing subscriber");
            }
        });
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
