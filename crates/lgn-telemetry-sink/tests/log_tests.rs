use lgn_telemetry_sink::stream_info::get_stream_info;
use lgn_tracing::logs::LogStream;
use std::collections::HashMap;

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
