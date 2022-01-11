use log::debug;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn debug(message: &str) {
    set_panic_hook();

    debug!("Got a message from app: {}", message);
}

#[wasm_bindgen(start)]
pub fn main() {
    set_panic_hook();

    wasm_logger::init(wasm_logger::Config::default());
}
