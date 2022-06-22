use crate::components::GraphDefinition;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};

pub(crate) fn graph_update(mut graphs: Query<'_, '_, &mut GraphDefinition>, time: Res<'_, Time>) {
    let delta_time = time.delta_seconds();
    for mut graph in graphs.iter_mut() {
        graph.update(delta_time);
    }
    drop(graphs);
    drop(time);
}
