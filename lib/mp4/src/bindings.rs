#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MP4E_mux_tag {
    _unused: [u8; 0],
}

pub type MP4E_mux_t = MP4E_mux_tag;
//pub const track_media_kind_t_e_audio: track_media_kind_t = 0;
pub const track_media_kind_t_e_video: track_media_kind_t = 1;
//pub const track_media_kind_t_e_private: track_media_kind_t = 2;

pub type track_media_kind_t = ::std::os::raw::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MP4E_track_t {
    pub object_type_indication: ::std::os::raw::c_uint,
    pub language: [::std::os::raw::c_uchar; 4usize],
    pub track_media_kind: track_media_kind_t,
    pub time_scale: ::std::os::raw::c_uint,
    pub default_duration: ::std::os::raw::c_uint,
    pub u: MP4E_track_t_audio_video,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union MP4E_track_t_audio_video {
    pub a: MP4E_track_t_audio,
    pub v: MP4E_track_t_video,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MP4E_track_t_audio {
    pub channelcount: ::std::os::raw::c_uint,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MP4E_track_t_video {
    pub width: ::std::os::raw::c_int,
    pub height: ::std::os::raw::c_int,
}

extern "C" {
    // Allocates and initialize mp4 multiplexor
    // Given file handler is transparent to the MP4 library, and used only as
    // argument for given fwrite_callback() function.  By appropriate definition
    // of callback function application may use any other file output API (for
    // example C++ streams, or Win32 file functions)
    //
    // return multiplexor handle on success; NULL on failure
    pub fn MP4E_open(
        sequential_mode_flag: ::std::os::raw::c_int,
        enable_fragmentation: ::std::os::raw::c_int,
        token: *mut ::std::os::raw::c_void,
        write_callback: ::std::option::Option<
            unsafe extern "C" fn(
                offset: i64,
                buffer: *const ::std::os::raw::c_void,
                size: usize,
                token: *mut ::std::os::raw::c_void,
            ) -> ::std::os::raw::c_int,
        >,
    ) -> *mut MP4E_mux_t;
}
extern "C" {
    // Add new track
    // The track_data parameter does not referred by the multiplexer after function
    // return, and may be allocated in short-time memory. The dsi member of
    // track_data parameter is mandatory.
    //
    // return ID of added track, or error code MP4E_STATUS_*
    pub fn MP4E_add_track(
        mux: *mut MP4E_mux_t,
        track_data: *const MP4E_track_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    // Add new sample to specified track
    // The tracks numbered starting with 0, according to order of MP4E_add_track() calls
    // 'kind' is one of MP4E_SAMPLE_... defines
    //
    // return error code MP4E_STATUS_*
    //
    // Example:
    //     MP4E_put_sample(mux, 0, data, data_bytes, duration, MP4E_SAMPLE_DEFAULT);
    pub fn MP4E_put_sample(
        mux: *mut MP4E_mux_t,
        track_num: ::std::os::raw::c_int,
        data: *const ::std::os::raw::c_void,
        data_bytes: ::std::os::raw::c_int,
        duration: ::std::os::raw::c_int,
        kind: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
//extern "C" {
//    // Finalize MP4 file, de-allocated memory, and closes MP4 multiplexer.
//    // The close operation takes a time and disk space, since it writes MP4 file
//    // indexes.  Please note that this function does not closes file handle,
//    // which was passed to open function.
//    //
//    // return error code MP4E_STATUS_*
//    pub fn MP4E_close(mux: *mut MP4E_mux_t) -> ::std::os::raw::c_int;
//}
//extern "C" {
//    // Set Decoder Specific Info (DSI)
//    // Can be used for audio and private tracks.
//    // MUST be used for AAC track.
//    // Only one DSI can be set. It is an error to set DSI again
//    //
//    // return error code MP4E_STATUS_*
//    pub fn MP4E_set_dsi(
//        mux: *mut MP4E_mux_t,
//        track_id: ::std::os::raw::c_int,
//        dsi: *const ::std::os::raw::c_void,
//        bytes: ::std::os::raw::c_int,
//    ) -> ::std::os::raw::c_int;
//}
//extern "C" {
//    // Set VPS data. MUST be used for HEVC (H.265) track.
//    //
//    // return error code MP4E_STATUS_*
//    pub fn MP4E_set_vps(
//        mux: *mut MP4E_mux_t,
//        track_id: ::std::os::raw::c_int,
//        vps: *const ::std::os::raw::c_void,
//        bytes: ::std::os::raw::c_int,
//    ) -> ::std::os::raw::c_int;
//}
extern "C" {
    // Set SPS data. MUST be used for AVC (H.264) track. Up to 32 different SPS can be used in one track.
    //
    // return error code MP4E_STATUS_*
    pub fn MP4E_set_sps(
        mux: *mut MP4E_mux_t,
        track_id: ::std::os::raw::c_int,
        sps: *const ::std::os::raw::c_void,
        bytes: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    // Set PPS data. MUST be used for AVC (H.264) track. Up to 256 different PPS can be used in one track.
    //
    // return error code MP4E_STATUS_*
    pub fn MP4E_set_pps(
        mux: *mut MP4E_mux_t,
        track_id: ::std::os::raw::c_int,
        pps: *const ::std::os::raw::c_void,
        bytes: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
//extern "C" {
//    // Set or replace ASCII test comment for the file. Set comment to NULL to remove comment.
//    //
//    // return error code MP4E_STATUS_*
//    pub fn MP4E_set_text_comment(
//        mux: *mut MP4E_mux_t,
//        comment: *const ::std::os::raw::c_char,
//    ) -> ::std::os::raw::c_int;
//}
