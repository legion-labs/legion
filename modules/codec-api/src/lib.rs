//! hw-codec create exposes the different hw codecs with the same interface
//!
//! The easiest way to the use the encoder is to create a pipeline
//! where you will get an input and output object, these object can be moved
//! to the context where they will be used, for example when encoding,
//! the renderer will own the input end of the pipeline, and the the streamer
//! will own the output end.
//!
//! ```
//! # use legion_codec_api::{
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

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]
#![warn(missing_docs)]

use std::sync::Arc;

/// Contains the hardware implementation of multiple encoding/decoding algorithms
pub mod backends;

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
        if let Some(video_processor) = Self::new(config) {
            let arc = Arc::new(video_processor);
            Some((Input::new(arc.clone()), Output::new(arc)))
        } else {
            None
        }
    }
}
