use crate::animation_graph_component::AnimationGraphComponent;

pub trait AnimationSystem {
    fn register_component() {
        /* */
    }

    fn unregister_component() {
        /* */
    }

    fn update() {
        /* */
    }

    fn update_anim_players() {
        /* */
    }

    fn update_anim_graphs() {
        /* */
    }
    // const anim_players: Vec<AnimationClipPlayerComponent>;
    const anim_graphs: Vec<AnimationGraphComponent>;
    // const mesh_components: Vec<SkeletalMeshComponents>; // TODO: find in our source code!
}
