use crate::components::GraphDefinition2;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};

pub(crate) fn graph_update(mut graphs: Query<'_, '_, &mut GraphDefinition>, time: Res<'_, Time>) {
    for mut graph in graphs.iter_mut() {
        let delta_time = time.delta_seconds();
        let current_node_index = graph.current_node_index;

        // update the graph
        // Todo!

        // update the current node
        (*graph.nodes[current_node_index as usize]).update(delta_time);
    }
    drop(graphs);
    drop(time);
}

pub(crate) fn clip_update(mut graphs: Query<'_, '_, &mut GraphDefinition2>, time: Res<'_, Time>) {
    drop(time);
}
