//! Legion Logger, implement the `Log` trait from the `log` crate
//! allows the use of the telemetry logging, and forwards the logs
//! to other loggers that implement the same trait
//! The goal of this crate is to provide some sendible defaults to legion apps
//! and the ability to configure logging

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::vec;

use lgn_telemetry::{log_static_str, log_string, LogLevel};
use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Record, SetLoggerError};
use simplelog::{ColorChoice, TermLogger, TerminalMode};

/// Legion Logger configuration
// todo: add hashmap for config per category
pub struct Config {
    /// global maximum log filter
    level_filter: LevelFilter,
    /// enable the default terminal logger
    terminal_logger: bool,
    /// any additional loggers implementing the `log::Log` trait
    /// these will be moved to the `Logger`
    additional_loggers: Vec<Box<dyn Log>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level_filter: LevelFilter::Info,
            terminal_logger: true,
            additional_loggers: vec![],
        }
    }
}

pub struct Logger {
    level_filter: LevelFilter,
    sub_loggers: Vec<Box<dyn Log>>,
}

impl Logger {
    /// Initializes the global logger
    ///
    /// # Errors
    /// Errors if a logger is already set
    ///
    /// # Examples
    /// ```
    /// use lgn_logger::*;
    /// use log::*;
    ///
    /// Logger::init(Config::default()).unwrap();
    /// info!("info log");
    /// ```
    pub fn init(config: Config) -> Result<(), SetLoggerError> {
        set_max_level(config.level_filter);
        let logger = Self::new(config);
        set_boxed_logger(logger)
    }

    /// Creates a new logger, .
    ///
    /// no macros are provided for this case and you probably
    /// dont want to use this function, but `init()`, if you dont want to build a `CombinedLogger`.
    ///
    ///
    /// # Examples
    /// ```
    /// use lgn_logger::*;
    /// use log::*;
    ///
    /// let _ = Logger::new(Config::default());
    /// ```
    pub fn new(config: Config) -> Box<Self> {
        let mut sub_loggers: Vec<Box<dyn Log>> = Vec::new();
        if config.terminal_logger {
            sub_loggers.push(TermLogger::new(
                config.level_filter,
                simplelog::Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ));
        }
        for sub_logger in config.additional_loggers {
            sub_loggers.push(sub_logger);
        }
        Box::new(Self {
            level_filter: config.level_filter,
            sub_loggers,
        })
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record<'_>) {
        let metadata = record.metadata();
        if self.enabled(metadata) {
            let level = LogLevel::from(record.level());
            if let Some(static_str) = record.args().as_str() {
                log_static_str(level, static_str);
            } else {
                log_string(
                    LogLevel::from(record.level()),
                    format!("target={} {}", metadata.target(), record.args()),
                );
            }
            for sub_logger in &self.sub_loggers {
                sub_logger.log(record);
            }
        }
    }

    fn flush(&self) {
        for sub_logger in &self.sub_loggers {
            sub_logger.flush();
        }
    }
}
