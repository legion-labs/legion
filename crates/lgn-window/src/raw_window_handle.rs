#![allow(unsafe_code)]

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct HasRawWindowHandleWrapper(RawWindowHandle);

// SAFE: the caller has validated that this is a valid context to get
// RawWindowHandle
unsafe impl HasRawWindowHandle for HasRawWindowHandleWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}
