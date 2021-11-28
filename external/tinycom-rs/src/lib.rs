// Copyright (c) 2016 com-rs developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//!
//! # tinycom-rs 0.1.0
//! Tiny Rust bindings for the Win32
//! [Component Object Model](https://msdn.microsoft.com/en-us/library/ms680573.aspx).
//!
//! # Overview
//! This crate is composed of three main components:
//!
//! * The [`com_interface!`](macro.com_interface!.html) macro for
//!   defining new interface types.
//! * The [`ComPtr`](struct.ComPtr.html) type for making use of them.
//! * Definition of [`IUnknown`](struct.IUnknown.html), the base COM interface.
//!

// TODO:
// * Tests for IUnknown/ComPtr, hard to test with no way of acquiring
//   IUnknown objects directly.

#![deny(dead_code)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod comptr;
mod iid;
mod unknown;

pub use comptr::{AsComPtr, ComInterface, ComPtr};
pub use iid::IID;
pub use unknown::{IID_IUnknown, IUnknown};

/// Result type.
pub type HResult = i32;
