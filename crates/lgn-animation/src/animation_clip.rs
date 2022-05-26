#![allow(dead_code)]

use crate::{animation_frame_time::FrameTime, animation_pose::Pose};
use lgn_transform::components::Transform;

use crate::{
    animation_event::Event, animation_skeleton::Skeleton, animation_sync_track::SyncTrack,
};

pub struct QuantizationRange {
    range_start: f32,
    range_length: f32,
}

impl QuantizationRange {
    #[inline]
    fn is_valid() {}
}

pub struct TrackCompressionSettings {
    translation_range_x: QuantizationRange,
    translation_range_y: QuantizationRange,
    translation_range_z: QuantizationRange,
    scale_range_x: QuantizationRange,
    scale_range_y: QuantizationRange,
    scale_range_z: QuantizationRange,
    track_start_index: u32,
    is_translation_static: bool,
    is_scale_static: bool,
}

impl TrackCompressionSettings {}

pub struct AnimationClip {
    skeleton: Skeleton,
    num_frames: u32,
    duration: f32,
    // compressed_pose_data: Vec<u16>, // Check if should start compressed or not
    // track_compression_settings: Vec<TrackCompressionSettings>,
    track_data: u16,
    root_motion_track: Vec<Transform>,
    events: Vec<Event>,
    sync_track: SyncTrack,
    average_linear_velocity: f32,
    average_angular_velocity: f32,
    total_root_motion_delta: Transform,
    is_additive: bool,
}

impl AnimationClip {
    pub fn get_pose(&self, frame_time: FrameTime, out_pose: &Pose) {
        assert!(frame_time.frame_index < self.num_frames);

        let mut bone_transform = Transform::identity();

        let num_bones = self.skeleton.get_num_bones();
        for bone_idx in 0..num_bones {
            // bone_transform // !Todo
            out_pose.set_transform(bone_idx, bone_transform);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.num_frames > 0
    }

    #[inline]
    pub fn get_num_bones() {}

    #[inline]
    pub fn is_single_frame_animation() {}

    #[inline]
    pub fn get_fps() {}

    #[inline]
    pub fn get_time() { // returns seconds
    }

    #[inline]
    pub fn get_percentage_through() { // returns a percentage
    }

    pub fn get_frame_time() {}

    #[inline]
    pub fn get_local_space_transform() {}

    #[inline]
    pub fn get_global_space_transform() {}

    #[inline]
    pub fn get_root_transform() {}

    // Get the delta for the root motion for the given time range. Handle's looping but assumes only a single loop occurred.
    #[inline]
    pub fn get_root_motion_delta() {}

    #[inline]
    pub fn get_root_motion_delta_no_looping() {}

    #[inline]
    pub fn get_average_linear_velocity() {}

    #[inline]
    pub fn get_displacement_delta() {}

    #[inline]
    pub fn get_rotation_delta() {}

    // Get all the events for the specified range. This function will append the results to the output array. Handle's looping but assumes only a single loop occurred.
    #[inline]
    pub fn get_events_for_range() {}

    // Get all the events for the specified range. This function will append the results to the output array. DOES NOT SUPPORT LOOPING!
    #[inline]
    pub fn get_events_for_range_no_looping() {}

    #[inline]
    pub fn read_compressed_track_transform() {}

    #[inline]
    pub fn read_compressed_track_key_frame() {}
}
