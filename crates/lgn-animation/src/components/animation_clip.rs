use crate::animation_skeleton::Skeleton;
use crate::runtime::{AnimationTrack, VecAnimationTransform};

use lgn_ecs::component::Component;
use lgn_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

#[derive(Component, Clone)]
pub struct AnimationClip {
    pub current_key_frame_index: i32,
    pub duration_key_frames: Vec<f32>,
    pub time_since_last_tick: f32,
    pub looping: bool,
    pub skeleton: Skeleton,
}

impl AnimationClip {
    #[must_use]
    pub fn new(raw_animation_track: &AnimationTrack) -> Self {
        let converted_poses = convert_raw_pose_data(&raw_animation_track.key_frames);
        let mut skeleton: Skeleton = Skeleton {
            bone_ids: raw_animation_track.bone_ids.clone(),
            parent_indices: raw_animation_track.parent_indices.clone(),
            poses: converted_poses,
        };
        update_children_transforms(&mut skeleton);
        Self {
            current_key_frame_index: raw_animation_track.current_key_frame_index,
            duration_key_frames: raw_animation_track.duration_key_frames.clone(),
            time_since_last_tick: raw_animation_track.time_since_last_tick,
            looping: raw_animation_track.looping,
            skeleton,
        }
    }
}

pub(crate) fn convert_raw_pose_data(
    raw_poses: &Vec<VecAnimationTransform>,
) -> Vec<Vec<TransformBundle>> {
    let mut poses: Vec<Vec<TransformBundle>> = Vec::new();
    for vec_anim_transform in raw_poses {
        let mut vec_transform_bundle: Vec<TransformBundle> = Vec::new();
        for anim_transform_bundle in &vec_anim_transform.anim_transform_vec {
            vec_transform_bundle.push(TransformBundle {
                local: Transform {
                    translation: anim_transform_bundle.local.translation,
                    rotation: anim_transform_bundle.local.rotation,
                    scale: anim_transform_bundle.local.scale,
                },
                global: GlobalTransform::identity(),
            });
        }
        poses.push(vec_transform_bundle);
    }
    poses
}

pub(crate) fn update_children_transforms(skeleton: &mut Skeleton) {
    for n_pose in 0..skeleton.poses.len() {
        for n_bone in 0..skeleton.bone_ids.len() {
            if !is_root_bone(skeleton.parent_indices[n_bone]) {
                skeleton.poses[n_pose][n_bone].global = skeleton.poses[n_pose]
                    [skeleton.parent_indices[n_bone] as usize]
                    .global
                    .mul_transform(skeleton.poses[n_pose][n_bone].local);
            } else {
                skeleton.poses[n_pose][n_bone].global = skeleton.poses[n_pose][n_bone].local.into();
            }
        }
    }
}

pub(crate) fn is_root_bone(parent_idx: i32) -> bool {
    parent_idx == -1
}

// #![allow(dead_code)]

// use crate::{animation_frame_time::FrameTime, animation_pose::Pose};
// use lgn_transform::components::Transform;

// use crate::{
//     animation_event::Event, animation_skeleton::Skeleton, animation_sync_track::SyncTrack,
// };

// pub struct QuantizationRange {
//     range_start: f32,
//     range_length: f32,
// }

// impl QuantizationRange {
//     #[inline]
//     fn is_valid() {}
// }

// pub struct TrackCompressionSettings {
//     translation_range_x: QuantizationRange,
//     translation_range_y: QuantizationRange,
//     translation_range_z: QuantizationRange,
//     scale_range_x: QuantizationRange,
//     scale_range_y: QuantizationRange,
//     scale_range_z: QuantizationRange,
//     track_start_index: u32,
//     is_translation_static: bool,
//     is_scale_static: bool,
// }

// impl TrackCompressionSettings {}

// pub struct AnimationClip {
//     skeleton: Skeleton,
//     num_frames: u32,
//     duration: f32,
//     // compressed_pose_data: Vec<u16>, // Check if should start compressed or not
//     // track_compression_settings: Vec<TrackCompressionSettings>,
//     // track_data: u16,
//     // root_motion_track: Vec<Transform>,
//     // events: Vec<Event>,
//     // sync_track: SyncTrack,
//     // average_linear_velocity: f32,
//     // average_angular_velocity: f32,
//     // total_root_motion_delta: Transform,
//     // is_additive: bool,
// }

// impl AnimationClip {
//     pub fn get_pose(&self, frame_time: FrameTime, out_pose: &Pose) {
//         assert!(frame_time.frame_index < self.num_frames);

//         let mut bone_transform = Transform::identity();

//         let num_bones = self.skeleton.get_num_bones();
//         for bone_idx in 0..num_bones {
//             // bone_transform // !Todo
//             out_pose.set_transform(bone_idx, bone_transform);
//         }
//     }

//     pub fn is_valid(&self) -> bool {
//         self.num_frames > 0
//     }

//     #[inline]
//     pub fn get_num_bones() {}

//     #[inline]
//     pub fn is_single_frame_animation() {}

//     #[inline]
//     pub fn get_fps() {}

//     #[inline]
//     pub fn get_time() { // returns seconds
//     }

//     #[inline]
//     pub fn get_percentage_through() { // returns a percentage
//     }

//     pub fn get_frame_time() {}

//     #[inline]
//     pub fn get_local_space_transform() {}

//     #[inline]
//     pub fn get_global_space_transform() {}

//     #[inline]
//     pub fn get_root_transform() {}

//     // Get the delta for the root motion for the given time range. Handle's looping but assumes only a single loop occurred.
//     #[inline]
//     pub fn get_root_motion_delta() {}

//     #[inline]
//     pub fn get_root_motion_delta_no_looping() {}

//     #[inline]
//     pub fn get_average_linear_velocity() {}

//     #[inline]
//     pub fn get_displacement_delta() {}

//     #[inline]
//     pub fn get_rotation_delta() {}

//     // Get all the events for the specified range. This function will append the results to the output array. Handle's looping but assumes only a single loop occurred.
//     #[inline]
//     pub fn get_events_for_range() {}

//     // Get all the events for the specified range. This function will append the results to the output array. DOES NOT SUPPORT LOOPING!
//     #[inline]
//     pub fn get_events_for_range_no_looping() {}

//     #[inline]
//     pub fn read_compressed_track_transform() {}

//     #[inline]
//     pub fn read_compressed_track_key_frame() {}
// }
