use crate::*;
use std::sync::Arc;

// TelemetrySinkEvent are sent when something is worth 'writing home about'
//  i.e. writing it to disk or sending it to the server
#[derive(Debug)]
pub enum TelemetrySinkEvent {
    OnInitProcess(ProcessInfo),
    OnInitStream(StreamInfo),
    OnLogBufferFull(Arc<LogBlock>),
    OnThreadBufferFull(Arc<ThreadBlock>),
    OnShutdown,
}

pub trait EventBlockSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent);
}

pub struct NullEventSink {}
impl EventBlockSink for NullEventSink {
    fn on_sink_event(&self, _event: TelemetrySinkEvent) {}
}
