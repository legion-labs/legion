//! A very basic example of a "wasm crate".
//! Binds the existing `console.log` JavaScript function,
//! and exposes a simple `debug` function.

#![no_std]
#![allow(clippy::unused_unit)]

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    #[allow(unsafe_code)]
    fn log(message: &str);
}

#[wasm_bindgen]
pub fn debug(message: &str) {
    log(message);
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    log("Initializing the wasm crate");
}
