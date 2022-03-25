use lgn_analytics::log_entry_from_value;
use lgn_analytics::parse_block;
use lgn_telemetry_sink::stream_block::StreamBlock;
use lgn_telemetry_sink::stream_info::get_stream_info;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::event::TracingBlock;
use lgn_tracing::logs::LogBlock;
use lgn_tracing::logs::LogStaticStrInteropEvent;
use lgn_tracing::logs::LogStream;
use lgn_tracing::logs::LogStringInteropEvent;
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
        .position(|udt| udt.name == "LogStringInteropEventV2")
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
fn test_log_encode_static() {
    let _telemetry_guard = TelemetryGuard::default();
    let mut stream = LogStream::new(1024, String::from("bogus_process_id"), &[], HashMap::new());
    let stream_id = stream.stream_id().to_string();
    stream.get_events_mut().push(LogStaticStrInteropEvent {
        time: 1,
        level: 2,
        target: "target_name".into(),
        msg: "my message".into(),
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
            assert_eq!(&*obj.get::<Arc<String>>("target").unwrap(), "target_name");
            assert_eq!(&*obj.get::<Arc<String>>("msg").unwrap(), "my message");
        } else {
            panic!("log entry not an object");
        }
        Ok(true)
    })
    .unwrap();
}

#[test]
fn test_log_encode_dynamic() {
    let _telemetry_guard = TelemetryGuard::default();
    let mut stream = LogStream::new(1024, String::from("bogus_process_id"), &[], HashMap::new());
    let stream_id = stream.stream_id().to_string();
    stream.get_events_mut().push(LogStringInteropEvent {
        time: 1,
        level: 2,
        target: "target_name".into(),
        msg: lgn_tracing_transit::DynString(String::from("my message")),
    });
    let mut block = stream.replace_block(Arc::new(LogBlock::new(1024, stream_id)));
    Arc::get_mut(&mut block).unwrap().close();
    let encoded = block.encode().unwrap();
    assert_eq!(encoded.nb_objects, 1);
    let stream_info = get_stream_info(&stream);
    parse_block(&stream_info, &encoded.payload.unwrap(), |val| {
        if let Value::Object(obj) = val {
            assert_eq!(obj.type_name.as_str(), "LogStringInteropEventV2");
            assert_eq!(obj.get::<i64>("time").unwrap(), 1);
            assert_eq!(obj.get::<u32>("level").unwrap(), 2);
            assert_eq!(&*obj.get::<Arc<String>>("target").unwrap(), "target_name");
            assert_eq!(&*obj.get::<Arc<String>>("msg").unwrap(), "my message");
        } else {
            panic!("log entry not an object");
        }
        Ok(true)
    })
    .unwrap();
}

#[test]
fn test_parse_log_interops() {
    let _telemetry_guard = TelemetryGuard::default();
    let mut stream = LogStream::new(1024, String::from("bogus_process_id"), &[], HashMap::new());
    let stream_id = stream.stream_id().to_string();
    stream.get_events_mut().push(LogStaticStrInteropEvent {
        time: 1,
        level: 2,
        target: "target_name".into(),
        msg: "my message".into(),
    });
    stream.get_events_mut().push(LogStringInteropEvent {
        time: 1,
        level: 2,
        target: "target_name".into(),
        msg: lgn_tracing_transit::DynString(String::from("my message")),
    });
    let mut block = stream.replace_block(Arc::new(LogBlock::new(1024, stream_id)));
    Arc::get_mut(&mut block).unwrap().close();
    let encoded = block.encode().unwrap();
    assert_eq!(encoded.nb_objects, 2);
    let stream_info = get_stream_info(&stream);
    let mut nb_log_entries = 0;
    parse_block(&stream_info, &encoded.payload.unwrap(), |val| {
        if let Some((_time, _msg)) = log_entry_from_value(&val).unwrap() {
            nb_log_entries += 1;
        }
        Ok(true)
    })
    .unwrap();
    assert_eq!(nb_log_entries, 2);
}
