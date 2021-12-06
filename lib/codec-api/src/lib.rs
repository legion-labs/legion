//! hw-codec create exposes the different hw codecs with the same interface
//!
//! The easiest way to the use the encoder is to create a pipeline
//! where you will get an input and output object, these object can be moved
//! to the context where they will be used, for example when encoding,
//! the renderer will own the input end of the pipeline, and the the streamer
//! will own the output end.
//!
//! ```
//! # use lgn_codec_api::{
//! #    backends::null::{NullEncoder, NullEncoderConfig},
//! #    Error, GpuImage, VideoProcessor,
//! # };
//! # use std::thread;
//! let mut frame_count = 100;
//! let (input, output) =
//!     NullEncoder::pipeline(NullEncoderConfig { queue_size: 10 })
//!         .expect("NullEncoder should be valid");
//! let thread_handle = thread::spawn(move || {
//!     while frame_count > 0 {
//!         if output.query().is_ok() {
//!             frame_count -= 1;
//!         };
//!     }
//! });
//! while frame_count > 0 {
//!     match input.submit(&GpuImage::Vulkan(ash::vk::Image::null())) {
//!         Ok(_) => frame_count -= 1,
//!         Err(Error::BufferFull) => {}
//!         Err(_) => panic!("Unexpected error from the NullEncoder"),
//!     };
//! }
//! thread_handle
//!     .join()
//!     .expect("the receiver thread should exit properly");
//! ```

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(clippy::missing_errors_doc)]
//#![warn(missing_docs)]

use std::sync::Arc;

/// Contains the hardware implementation of multiple encoding/decoding algorithms
pub mod backends;

pub mod formats;

/// doc
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// doc
    Eof,
    /// doc
    Repeat,
    /// doc
    BufferFull,
    /// doc
    NeedInputs,
    /// doc
    Failed(&'static str),
}

/// Gpu Image handle either outputted or sent to a video processor
pub enum GpuImage {
    /// Vulkan image
    Vulkan(ash::vk::Image),
}

/// Cpu buffer handle either outputted or sent to a video processor
pub struct CpuBuffer(Vec<u8>);

/// Gpu buffer handle either outputted or sent to a video processor
pub enum GpuBuffer {
    /// doc
    Vulkan(ash::vk::Buffer),
}

/// Input end of the pipe allowing you
pub struct Input<VP: VideoProcessor> {
    video_processor: Arc<VP>,
}

impl<VP: VideoProcessor> Input<VP> {
    fn new(video_processor: Arc<VP>) -> Self {
        Self { video_processor }
    }

    /// submit a an input, operation should not be blocking
    /// some errors needs to be handled
    pub fn submit(&self, a: &VP::Input) -> Result<(), Error> {
        self.video_processor.submit_input(a)
    }
}

/// Output end of a pipeline
pub struct Output<VP: VideoProcessor> {
    video_processor: Arc<VP>,
}

impl<VP: VideoProcessor> Output<VP> {
    fn new(video_processor: Arc<VP>) -> Self {
        Self { video_processor }
    }

    /// Query output, can be blocking
    pub fn query(&self) -> Result<VP::Output, Error> {
        self.video_processor.query_output()
    }
}

/// Video Processor trait, implemented by encoders and decoders
pub trait VideoProcessor: Sized + Send + Sync {
    /// Input type, like `GpuImage`, `GpuBuffer`, `CpuBuffer`
    type Input;
    /// Output type, like `GpuImage`, `GpuBuffer`, `CpuBuffer`
    type Output;
    /// Config type, like `AmfEncoderConfig`
    type Config;

    /// Create a new instance of a concrete video processor if possible
    /// Given that `VideoProcessors` are hardware bound, it is possible
    /// not to be able to create a Encoder given a Config, like asking for
    /// an Amf encoder on Nvidia hardware, or requesting an VP9 encoder
    /// on AMD hardware
    fn new(config: Self::Config) -> Option<Self>;

    /// submit input sends an input to a video processor
    /// function might mutate it's interior state, so calling in another thread
    /// from query output is possible and even recommended
    fn submit_input(&self, a: &Self::Input) -> Result<(), Error>;

    /// Query an output
    fn query_output(&self) -> Result<Self::Output, Error>;

    /// doc
    fn pipeline(config: Self::Config) -> Option<(Input<Self>, Output<Self>)> {
        Self::new(config).map(|video_processor| {
            let arc = Arc::new(video_processor);
            (Input::new(arc.clone()), Output::new(arc))
        })
    }
}
