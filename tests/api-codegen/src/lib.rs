//! Testing library for the api codegen.
//!
// crate-specific lint exceptions:
// #![allow()]

#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod api_impl;

lgn_online::include_api!();
