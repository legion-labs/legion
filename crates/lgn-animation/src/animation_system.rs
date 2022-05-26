use crate::components::GraphDefinition;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};

pub(crate) fn graph_update(mut graphs: Query<'_, '_, &mut GraphDefinition>, time: Res<'_, Time>) {
    for mut graph in graphs.iter_mut() {
        let delta_time = time.delta_seconds();
        let current_node_index = graph.current_node_index;

        // update the current node
        (*graph.nodes[current_node_index as usize]).update(delta_time);
    }
    drop(graphs);
    drop(time);
}

pub(crate) fn is_root_bone(parent_idx: i32) -> bool {
    parent_idx == -1
}

// !Todo If we need an animation system
// pub struct AnimationSystem {
//    skeleton: Skeleton,
//    animation_graph: GraphInstance,
//    animation_clip: AnimationClip,
// }

// impl AnimationSystem {
//     pub(crate) fn update_anim_players() {}
//     pub(crate) fn update_anim_graphs() {}

//     pub(crate) fn is_exact_key_frame(
//         &self,
//         time_since_last_tick: f32,
//         duration_current_key_frame: f32,
//     ) -> bool {
//         time_since_last_tick / duration_current_key_frame >= 1.0
//     }
// }
