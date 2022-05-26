use crate::animation_skeleton::Skeleton;
use crate::runtime::{AnimationTrack, VecAnimationTransform};

use lgn_ecs::component::Component;
use lgn_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

#[derive(Component, Clone)]
pub struct RuntimeAnimationTrack {
    pub current_key_frame_index: i32,
    pub duration_key_frames: Vec<f32>,
    pub time_since_last_tick: f32,
    pub looping: bool,
    pub skeleton: Skeleton,
    // nodes: Vec<Vec3>,
}

impl RuntimeAnimationTrack {
    pub fn new(raw_animation_track: &AnimationTrack) -> Self {
        let converted_poses = convert_raw_pose_data(&raw_animation_track.key_frames);
        Self {
            current_key_frame_index: raw_animation_track.current_key_frame_index,
            duration_key_frames: raw_animation_track.duration_key_frames.clone(),
            time_since_last_tick: raw_animation_track.time_since_last_tick,
            looping: raw_animation_track.looping,
            skeleton: Skeleton {
                bone_ids: raw_animation_track.bone_ids.clone(),
                parent_indices: raw_animation_track.parent_indices.clone(),
                poses: converted_poses,
            },
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
                global: GlobalTransform {
                    translation: anim_transform_bundle.global.translation,
                    rotation: anim_transform_bundle.global.rotation,
                    scale: anim_transform_bundle.global.scale,
                },
            });
        }
        poses.push(vec_transform_bundle);
    }
    poses
}
