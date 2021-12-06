#[test]
fn test_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        dbg!(panic_info);
        std::process::exit(0);
    }));
    lgn_telemetry::panic_hook::init_panic_hook();
    panic!("PANIC!!");
}
