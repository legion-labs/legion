//! hw-codec create exposes the different hw codecs with the same interface
//!
//! The easiest way to the use the encoder is to create a pipeline
//! where you will get an input and output object, these object can be moved
//! to the context where they will be used, for example when encoding,
//! the renderer will own the input end of the pipeline, and the the streamer
//! will own the output end.

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]
//#![warn(missing_docs)]

/// Contains the hardware implementation of multiple encoding/decoding
/// algorithms
pub mod backends;
pub mod formats;

pub mod encoder_resource;
pub mod stream_encoder;

/// doc
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Encoder '{encoder}' failed loading because '{reason}'")]
    Init {
        /// Encoder name
        encoder: &'static str,
        /// Reason for the failure
        reason: String,
    },
    #[error("End of stream")]
    Eof,
    #[error("Repeat last frame")]
    Repeat,
    #[error("Buffer full")]
    BufferFull,
    #[error("Need input")]
    NeedInputs,
    #[error("generic failure '{0}'")]
    Failed(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
