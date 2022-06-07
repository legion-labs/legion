use crate::components::AnimationClip;

#[derive(Clone)]
pub struct RuntimeAnimationClipNode {
    pub id: i32,
    pub clip: AnimationClip,
}
