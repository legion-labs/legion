//! Legion MP4 library, supports legion use cases of live streaming
//! as well as saving the stream to a file for post processing
//! The priority is put on the live streaming use case
//! Currently the using minmp4 under the hood, a pure rust representation
//! is under construction
//!

// BEGIN - Legion Labs lints v0.5
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow(unsafe_code)]

mod error;
pub use error::*;

mod atoms;

mod types;
pub use types::*;

mod track;
pub use track::*;

mod mse_writer;
pub use mse_writer::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mp4Config {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
    pub timescale: u32,
}

mod bindings;
pub mod old {
    use std::{convert::TryInto, os::raw::c_int};

    use crate::bindings;

    pub struct Mp4StreamInner {
        buffer: Vec<u8>,
        muxer: *mut bindings::MP4E_mux_t,
    }

    pub struct Mp4Stream {
        // created once per stream, boxed here for convenience
        inner: Box<Mp4StreamInner>,
        fps: i32,
    }

    unsafe impl Send for Mp4Stream {}
    unsafe impl Sync for Mp4Stream {}

    pub type Mp4Result<T> = Result<T, Mp4Error>;

    #[derive(Debug, Clone)]
    pub enum Mp4Error {
        BadArguments,
        NoMemory,
        WriteError,
        OnlyOneDSIAllowed,
        Undefined,
    }

    struct BindingsResult(c_int);

    impl From<BindingsResult> for Mp4Result<i32> {
        fn from(value: BindingsResult) -> Self {
            match value.0 {
                -1 => Err(Mp4Error::BadArguments),
                -2 => Err(Mp4Error::NoMemory),
                -3 => Err(Mp4Error::WriteError),
                -4 => Err(Mp4Error::OnlyOneDSIAllowed),
                c_int::MIN..=-5 => Err(Mp4Error::Undefined),
                i => Ok(i),
            }
        }
    }

    impl From<BindingsResult> for Mp4Result<()> {
        fn from(value: BindingsResult) -> Self {
            let res: Mp4Result<i32> = value.into();
            res.map(|_| ())
        }
    }

    impl Mp4Stream {
        pub fn new(fps: i32) -> Self {
            let mut inner = Box::new(Mp4StreamInner {
                buffer: vec![],
                muxer: std::ptr::null_mut(),
            });
            let muxer = unsafe {
                bindings::MP4E_open(
                    1,
                    1,
                    (inner.as_mut() as *mut Mp4StreamInner).cast::<std::ffi::c_void>(),
                    Some(write_callback),
                )
            };
            inner.muxer = muxer;
            Self { inner, fps }
        }

        /// # Errors
        pub fn add_track(&mut self, width: i32, height: i32) -> Mp4Result<i32> {
            let track_data = bindings::MP4E_track_t {
                object_type_indication: 0x21,
                language: [b'u', b'n', b'd', 0],
                track_media_kind: bindings::track_media_kind_t_e_video,
                time_scale: 90_000,
                default_duration: 0,
                u: bindings::MP4E_track_t_audio_video {
                    v: bindings::MP4E_track_t_video { width, height },
                },
            };
            unsafe {
                BindingsResult(bindings::MP4E_add_track(self.inner.muxer, &track_data)).into()
            }
        }

        /// # Errors
        pub fn set_sps(&mut self, track_id: i32, nalu: &[u8]) -> Mp4Result<()> {
            unsafe {
                BindingsResult(bindings::MP4E_set_sps(
                    self.inner.muxer,
                    track_id,
                    nalu.as_ptr().cast::<std::ffi::c_void>(),
                    nalu.len()
                        .try_into()
                        .expect("nalu size lower than i32::MAX"),
                ))
                .into()
            }
        }

        /// # Errors
        pub fn set_pps(&mut self, track_id: i32, nalu: &[u8]) -> Mp4Result<()> {
            unsafe {
                BindingsResult(bindings::MP4E_set_pps(
                    self.inner.muxer,
                    track_id,
                    nalu.as_ptr().cast::<std::ffi::c_void>(),
                    nalu.len()
                        .try_into()
                        .expect("nalu size lower than i32::MAX"),
                ))
                .into()
            }
        }

        /// # Errors
        pub fn add_frame(&mut self, track_id: i32, idr: bool, nalu: &[u8]) -> Mp4Result<()> {
            unsafe {
                BindingsResult(bindings::MP4E_put_sample(
                    self.inner.muxer,
                    track_id,
                    nalu.as_ptr().cast::<std::ffi::c_void>(),
                    nalu.len()
                        .try_into()
                        .expect("nalu size lower than i32::MAX"),
                    90000 / self.fps,
                    if idr { 1 } else { 0 },
                ))
                .into()
            }
        }

        pub fn get_content(&self) -> &[u8] {
            &self.inner.buffer
        }

        pub fn clean(&mut self) {
            self.inner.buffer.clear();
        }
    }

    unsafe extern "C" fn write_callback(
        _: i64,
        buffer: *const std::ffi::c_void,
        size: usize,
        token: *mut std::ffi::c_void,
    ) -> c_int {
        let inner = &mut *token.cast::<Mp4StreamInner>();
        let buffer = std::slice::from_raw_parts(buffer.cast::<u8>(), size);
        inner.buffer.extend_from_slice(buffer);
        0
    }
}
