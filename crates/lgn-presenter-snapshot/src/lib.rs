//! Presenter plugin made for windowing system.

// crate-specific lint exceptions:
//#![allow()]

pub mod component;

mod offscreen_capture;
use offscreen_capture::OffscreenHelper;
