//! Presenter plugin made for windowing system.

// crate-specific lint exceptions:
//#![allow()]

mod cgen {
    include!(concat!(env!("OUT_DIR"), "/rust/mod.rs"));
}
#[allow(unused_imports)]
use cgen::*;

pub mod component;

mod offscreen_capture;
use offscreen_capture::OffscreenHelper;
