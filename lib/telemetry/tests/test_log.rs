use telemetry::*;

#[test]
fn test_log_str() {
    init_event_dispatch(1024).unwrap();
    log_str(LogLevel::Info, "test");
}
