use lgn_analytics::parse_block;
use lgn_telemetry_sink::stream_block::StreamBlock;
use lgn_telemetry_sink::stream_info::get_stream_info;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::event::TracingBlock;
use lgn_tracing::logs::LogBlock;
use lgn_tracing::logs::LogStaticStrInteropEvent;
use lgn_tracing::logs::LogStream;
use lgn_tracing_transit::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_log_interop_metadata() {
    let stream = LogStream::new(1024, String::from("bogus_process_id"), &[], HashMap::new());
    let stream_proto = get_stream_info(&stream);
    let obj_meta = stream_proto.objects_metadata.unwrap();
    obj_meta
        .types
        .iter()
        .position(|udt| udt.name == "LogStringInteropEvent")
        .unwrap();
    obj_meta
        .types
        .iter()
        .position(|udt| udt.name == "LogStaticStrInteropEvent")
        .unwrap();
    obj_meta
        .types
        .iter()
        .position(|udt| udt.name == "StringId")
        .unwrap();
}

#[test]
fn test_log_encode() {
    let _telemetry_guard = TelemetryGuard::default();
    let mut stream = LogStream::new(1024, String::from("bogus_process_id"), &[], HashMap::new());
    let stream_id = stream.stream_id().to_string();
    stream.get_events_mut().push(LogStaticStrInteropEvent {
        time: 1,
        level: 2,
        target: "target".into(),
        msg: "msg".into(),
    });
    let mut block = stream.replace_block(Arc::new(LogBlock::new(1024, stream_id)));
    Arc::get_mut(&mut block).unwrap().close();
    let encoded = block.encode().unwrap();
    assert_eq!(encoded.nb_objects, 1);
    let stream_info = get_stream_info(&stream);
    parse_block(&stream_info, &encoded.payload.unwrap(), |val| {
        if let Value::Object(obj) = val {
            assert_eq!(obj.type_name.as_str(), "LogStaticStrInteropEvent");
            assert_eq!(obj.get::<i64>("time").unwrap(), 1);
            assert_eq!(obj.get::<u32>("level").unwrap(), 2);
            dbg!(obj);
        } else {
            panic!("log entry not an object");
        }
        Ok(true)
    })
    .unwrap();
}
