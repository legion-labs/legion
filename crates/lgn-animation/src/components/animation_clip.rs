use crate::animation_pose::Pose;
use crate::animation_skeleton::Skeleton;
use crate::runtime::{AnimationTrack, AnimationTransformBundleVec};

use lgn_ecs::component::Component;
use lgn_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

#[derive(Component, Clone)]
pub struct RuntimeAnimationClip {
    pub(crate) current_key_frame_index: u32,
    pub(crate) duration_key_frames: Vec<f32>,
    pub(crate) time_since_last_tick: f32,
    pub(crate) looping: bool,
    pub(crate) poses: Vec<Pose>,
}

impl RuntimeAnimationClip {
    #[must_use]
    pub fn new(raw_animation_track: &AnimationTrack) -> Self {
        let skeleton = Skeleton {
            bone_ids: raw_animation_track.bone_ids.clone(),
            parent_indices: raw_animation_track.parent_indices.clone(),
        };
        let converted_transforms = convert_raw_pose_data(&raw_animation_track.key_frames);
        let mut converted_poses = Vec::new();

        for pose_transforms in converted_transforms {
            converted_poses.push(Pose {
                skeleton: skeleton.clone(),
                transforms: pose_transforms,
            });
        }

        update_children_transforms(&mut converted_poses);

        Self {
            current_key_frame_index: raw_animation_track.current_key_frame_index,
            duration_key_frames: raw_animation_track.duration_key_frames.clone(),
            time_since_last_tick: raw_animation_track.time_since_last_tick,
            looping: raw_animation_track.looping,
            poses: converted_poses,
        }
    }
}

fn convert_raw_pose_data(
    raw_poses: &Vec<AnimationTransformBundleVec>,
) -> Vec<Vec<TransformBundle>> {
    let mut poses: Vec<Vec<TransformBundle>> = Vec::new();
    for pose in raw_poses {
        let mut vec_transform_bundle = Vec::new();
        for anim_transform_bundle in &pose.anim_transform_vec {
            vec_transform_bundle.push(TransformBundle {
                local: Transform {
                    translation: anim_transform_bundle.translation,
                    rotation: anim_transform_bundle.rotation,
                    scale: anim_transform_bundle.scale,
                },
                global: GlobalTransform::identity(),
            });
        }
        poses.push(vec_transform_bundle);
    }
    poses
}

fn update_children_transforms(poses: &mut Vec<Pose>) {
    for pose in poses {
        pose.calculate_global_transforms();
    }
}
