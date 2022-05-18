#![allow(dead_code)]

// use crate::animation_graph_component::AnimationGraphComponent;
use lgn_ecs::prelude::{Entity, Query};

// pub struct AnimationSystem {
//     // const anim_players: Vec<AnimationClipPlayerComponent>;
//     anim_graphs: Vec<AnimationGraphComponent>,
//     // const mesh_components: Vec<SkeletalMeshComponents>; // TODO: find in our source code!
// }

pub(crate) fn register_component() {
    /* */
}

pub(crate) fn unregister_component() {
    /* */
}

pub(crate) fn update(animations: Query<'_, '_, (Entity, &crate::runtime::AnimationComponent)>) {
    for (entity, animations) in animations.iter() {}
    drop(animations);
}

pub(crate) fn update_anim_players() {
    /* */
}

pub(crate) fn update_anim_graphs() {
    /* */
}
