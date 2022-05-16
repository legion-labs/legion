use lgn_math::{Quat, Vec4};
use lgn_transform::components::Transform;

use crate::{
    animation_event::Event, animation_frame_time::FrameTime, animation_skeleton::Skeleton,
    animation_sync_track::SyncTrack,
};

pub struct QuantizationRange {
    range_start: f32,
    range_length: f32,
}

impl QuantizationRange {
    #[inline]
    fn is_valid() -> bool {
        return false;
    }
}

pub struct TrackCompressionSettings {
    translation_range_X: QuantizationRange, // TODO QUantizationRange
    translation_range_Y: QuantizationRange,
    translation_range_Z: QuantizationRange,
    scale_range_X: QuantizationRange,
    scale_range_Y: QuantizationRange,
    scale_range_Z: QuantizationRange,
    track_start_index: u32,
    is_translation_static: bool,
    is_scale_static: bool,
}

pub struct AnimationClip {
    skeleton: Skeleton,
    num_frames: u32,
    duration: f32,
    compressed_pose_data: Vec<u16>, // Check if should start compressed or not
    track_compression_settings: Vec<TrackCompressionSettings>,
    root_motion_track: Vec<Transform>,
    events: Vec<Event>,
    sync_track: SyncTrack,
    average_linear_velocity: f32,
    average_angular_velocity: f32,
    total_root_motion_delta: Transform,
    is_additive: bool,
}

impl AnimationClip {
    #[inline]
    pub fn decode_rotation() {
        /* */
    }

    #[inline]
    pub fn decode_translation() {
        /* */
    }

    #[inline]
    pub fn decode_scale() {
        /* */
    }

    pub fn is_valid() {
        /* */
    }

    #[inline]
    pub fn get_skeleton() {
        /* */
    }

    #[inline]
    pub fn get_num_bones() {
        /* */
    }

    #[inline]
    pub fn is_single_frame_animation() {
        /* */
    }

    #[inline]
    pub fn get_fps() {
        /* */
    }

    #[inline]
    pub fn get_time() { // returns seconds
                        /* */
    }

    #[inline]
    pub fn get_percentage_through() { // returns a percentage
                                      /* */
    }

    pub fn get_frame_time() {
        /* */
    }

    pub fn get_pose() {
        /* */
    }

    #[inline]
    pub fn get_local_space_transform() {
        /* */
    }

    #[inline]
    pub fn get_global_space_transform() {
        /* */
    }

    #[inline]
    pub fn get_root_transform() {
        /* */
    }

    // Get the delta for the root motion for the given time range. Handle's looping but assumes only a single loop occurred.
    #[inline]
    pub fn get_root_motion_delta() {
        /* */
    }

    #[inline]
    pub fn get_root_motion_delta_no_looping() {
        /* */
    }

    #[inline]
    pub fn get_average_linear_velocity() {
        /* */
    }

    #[inline]
    pub fn get_displacement_delta() {
        /* */
    }

    #[inline]
    pub fn get_rotation_delta() {
        /* */
    }

    // Get all the events for the specified range. This function will append the results to the output array. Handle's looping but assumes only a single loop occurred.
    #[inline]
    pub fn get_events_for_range() {
        /* */
    }

    // Get all the events for the specified range. This function will append the results to the output array. DOES NOT SUPPORT LOOPING!
    #[inline]
    pub fn get_events_for_range_no_looping() {
        /* */
    }

    #[inline]
    pub fn read_compressed_track_transform() {
        /* */
    }

    #[inline]
    pub fn read_compressed_track_key_frame() {
        /* */
    }
}
