//! Log streaming plugin for Legion's engine.
//!
//! Will rebroadcast tracing events.

// crate-specific lint exceptions:
//#![allow()]

mod broadcast_sink;
mod grpc;
mod plugin;

pub use broadcast_sink::{BroadcastSink, TraceEvent};
pub use grpc::TraceEventsReceiver;
pub use plugin::LogStreamPlugin;
