use crate::animation_pose::Pose;
use crate::animation_skeleton::Skeleton;
use crate::runtime::{AnimationTrack, AnimationTransformBundleVec};
use lgn_ecs::component::Component;
use lgn_math::{Quat, Vec3};
use lgn_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};
use std::sync::Arc;

#[derive(Component, Clone)]
pub struct AnimationClip {
    pub(crate) name: String,
    pub(crate) current_key_frame_index: usize,
    pub(crate) duration_key_frames: Vec<f32>,
    pub(crate) time_since_last_tick: f32,
    pub(crate) looping: bool,
    pub(crate) poses: Vec<Pose>,
}

impl AnimationClip {
    #[must_use]
    pub fn new(raw_animation_track: &AnimationTrack, skeletons: &mut Vec<Arc<Skeleton>>) -> Self {
        let mut bone_ids: Vec<Option<usize>> = Vec::new();
        raw_animation_track
            .bone_ids
            .iter()
            .for_each(|bone_id| bone_ids.push(Some((*bone_id).try_into().unwrap())));
        let mut parent_indices: Vec<Option<usize>> = Vec::new();
        raw_animation_track
            .parent_indices
            .iter()
            .for_each(|parent_idx| {
                if *parent_idx >= 0 {
                    parent_indices.push(Some((*parent_idx).try_into().unwrap()));
                } else {
                    parent_indices.push(None);
                }
            });

        let mut idx = find_skeleton(skeletons, &parent_indices);
        if idx.is_none() {
            skeletons.push(Arc::new(Skeleton {
                bone_ids,
                parent_indices,
            }));
            idx = Some(skeletons.len() - 1);
        }

        let converted_transforms = convert_raw_pose_data(&raw_animation_track.key_frames);
        let mut converted_poses = Vec::new();

        for pose_transforms in converted_transforms {
            converted_poses.push(Pose {
                skeleton: Arc::clone(&skeletons[idx.unwrap()]),
                transforms: pose_transforms,
                root_motion: GlobalTransform {
                    translation: Vec3::new(0.4, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(0.0, 0.0, 0.0),
                },
                current_root_position: GlobalTransform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(0.0, 0.0, 0.0),
                },
            });
        }

        update_children_transforms(&mut converted_poses);

        Self {
            name: raw_animation_track.name.clone(),
            current_key_frame_index: raw_animation_track.current_key_frame_index,
            duration_key_frames: raw_animation_track.duration_key_frames.clone(),
            time_since_last_tick: raw_animation_track.time_since_last_tick,
            looping: raw_animation_track.looping,
            poses: converted_poses,
        }
    }
}

fn find_skeleton(
    skeletons: &[Arc<Skeleton>],
    parent_indices: &Vec<Option<usize>>,
) -> Option<usize> {
    let mut found = true;
    for (skeleton_id, skeleton) in skeletons.iter().enumerate() {
        if skeleton.parent_indices.len() != parent_indices.len() {
            found = false;
        } else {
            (0..skeleton.parent_indices.len()).for_each(|i| {
                if skeletons[skeleton_id].parent_indices[i] != parent_indices[i] {
                    found = false;
                }
            });
        }
        if found {
            return Some(skeleton_id);
        }
    }
    None
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
