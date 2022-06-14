use crate::components::GraphDefinition;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};

pub(crate) fn update(time: Res<'_, Time>, mut graphs: Query<'_, '_, &mut GraphDefinition>) {
    for mut graph in graphs.iter_mut() {
        let delta_time = time.delta_seconds();
        let current_node_index = graph.current_node_index as usize;

        graph.nodes[current_node_index].clip.time_since_last_tick += delta_time;

        let current_key_frame_idx = graph.nodes[current_node_index].clip.current_key_frame_index;

        // Changes pose when at exact key frame
        if is_exact_key_frame(
            graph.nodes[current_node_index].clip.time_since_last_tick,
            graph.nodes[current_node_index].clip.duration_key_frames
                [current_key_frame_idx as usize],
        ) {
            graph.nodes[current_node_index].clip.time_since_last_tick = 0.0;

            if graph.nodes[current_node_index].clip.looping
                && current_key_frame_idx
                    == graph.nodes[current_node_index].clip.poses.len() as u32 - 1
            {
                graph.nodes[current_node_index].clip.current_key_frame_index = 0;
                if graph.current_node_index == graph.nodes.len() as i32 - 1 {
                    graph.current_node_index = 0;
                } else {
                    graph.current_node_index += 1;
                }
            } else {
                graph.nodes[current_node_index].clip.current_key_frame_index += 1;
            }
        }
    }
    drop(graphs);
    drop(time);
}

fn is_exact_key_frame(time_since_last_tick: f32, duration_current_key_frame: f32) -> bool {
    time_since_last_tick >= duration_current_key_frame
}
