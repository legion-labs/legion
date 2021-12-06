use lgn_telemetry::*;
use LogMsgQueueAny::*;
use MetricsMsgQueueAny::*;
use ThreadEventQueueAny::*;

pub struct DebugEventSink {}

impl EventBlockSink for DebugEventSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent) {
        match event {
            TelemetrySinkEvent::OnInitProcess(_process_info) => {}
            TelemetrySinkEvent::OnInitStream(_stream_info) => {}
            TelemetrySinkEvent::OnLogBufferFull(log_buffer) => {
                for event in log_buffer.events.iter() {
                    match event {
                        LogMsgEvent(_evt) => {}
                        LogDynMsgEvent(_evt) => {}
                    }
                }
                if let Err(e) = log_buffer.encode() {
                    dbg!(e);
                }
            }
            TelemetrySinkEvent::OnThreadBufferFull(thread_buffer) => {
                for event in thread_buffer.events.iter() {
                    match event {
                        BeginScopeEvent(_evt) => {}
                        EndScopeEvent(_evt) => {}
                    }
                }
                if let Err(e) = thread_buffer.encode() {
                    dbg!(e);
                }
            }
            TelemetrySinkEvent::OnMetricsBufferFull(buffer) => {
                for event in buffer.events.iter() {
                    match event {
                        IntegerMetricEvent(_evt) => {}
                        FloatMetricEvent(_evt) => {}
                    }
                }
                if let Err(e) = buffer.encode() {
                    dbg!(e);
                }
            }
            TelemetrySinkEvent::OnShutdown => todo!(),
        }
    }
}
