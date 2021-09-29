//! Legion MP4 library, supports legion use cases of live streaming
//! as well as saving the stream to a file for post processing
//! The priority is put on the live streaming use case
//! Currently the using minmp4 under the hood, a pure rust representation
//! is under construction
//!

// BEGIN - Legion Labs lints v0.3
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
    rust_2018_idioms,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
// END - Legion Labs standard lints v0.3
// crate-specific exceptions:
#![allow(unsafe_code)]

use std::os::raw::c_int;

mod bindings;

pub struct Mp4StreamInner {
    buffer: Vec<u8>,
    muxer: *mut bindings::MP4E_mux_t,
}

pub struct Mp4Stream {
    // created once per stream, boxed here for conveignence
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
        unsafe { BindingsResult(bindings::MP4E_add_track(self.inner.muxer, &track_data)).into() }
    }

    pub fn set_sps(&mut self, track_id: i32, nalu: &[u8]) -> Mp4Result<()> {
        unsafe {
            BindingsResult(bindings::MP4E_set_sps(
                self.inner.muxer,
                track_id,
                nalu.as_ptr().cast::<std::ffi::c_void>(),
                nalu.len() as i32,
            ))
            .into()
        }
    }

    pub fn set_pps(&mut self, track_id: i32, nalu: &[u8]) -> Mp4Result<()> {
        unsafe {
            BindingsResult(bindings::MP4E_set_pps(
                self.inner.muxer,
                track_id,
                nalu.as_ptr().cast::<std::ffi::c_void>(),
                nalu.len() as i32,
            ))
            .into()
        }
    }

    pub fn add_frame(&mut self, track_id: i32, idr: bool, nalu: &[u8]) -> Mp4Result<()> {
        unsafe {
            BindingsResult(bindings::MP4E_put_sample(
                self.inner.muxer,
                track_id,
                nalu.as_ptr().cast::<std::ffi::c_void>(),
                nalu.len() as i32,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
