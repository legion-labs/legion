//! Legion MP4 library, supports legion use cases of live streaming
//! as well as saving the stream to a file for post processing
//! The priority is put on the live streaming use case
//! Currently the using minmp4 under the hood, a pure rust representation
//! is under construction

// crate-specific lint exceptions:
#![allow(unsafe_code)]

mod error;
pub use error::*;

mod atoms;

mod types;
pub use types::*;

mod track;
pub use track::*;

mod stream;
pub use stream::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mp4Config {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
    pub timescale: u32,
}
