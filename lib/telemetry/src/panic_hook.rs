use std::io::Write;
use std::panic::{take_hook, PanicInfo};

use crate::prelude::*;

pub fn init_panic_hook() {
    type BoxedHook = Box<dyn Fn(&PanicInfo<'_>) + Sync + Send + 'static>;
    static mut PREVIOUS_HOOK: Option<BoxedHook> = None;
    unsafe {
        assert!(PREVIOUS_HOOK.is_none());
        PREVIOUS_HOOK = Some(take_hook());
    }

    std::panic::set_hook(Box::new(|panic_info| unsafe {
        log_string(LogLevel::Error, format!("panic: {:?}", panic_info));
        shutdown_telemetry();
        if let Some(hook) = PREVIOUS_HOOK.as_ref() {
            std::io::stdout().flush().unwrap();
            hook(panic_info);
        }
    }));
}

#[allow(clippy::exit)]
pub fn init_ctrlc_hook() {
    ctrlc::set_handler(move || {
        log_static_str(LogLevel::Error, "ctrl-c");
        shutdown_telemetry();
        std::process::exit(1);
    })
    .expect("Error in ctrlc::set_handler");
}
