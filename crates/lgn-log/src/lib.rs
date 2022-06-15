//! Log streaming plugin for Legion's engine.
//!
//! Will rebroadcast tracing events.

// crate-specific lint exceptions:
//#![allow()]

mod api;
mod broadcast_sink;
mod plugin;
mod server;

pub use broadcast_sink::{BroadcastSink, TraceEvent};
pub use plugin::{LogStreamPlugin, TraceEventsReceiver};
