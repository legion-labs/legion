use crate::{animation_options::AnimationOptions, components::GraphDefinition};
use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::{Query, Res};
use lgn_graphics_data::Color;
use lgn_graphics_renderer::{debug_display::DebugDisplay, resources::DefaultMeshType};

pub(crate) fn display_animation(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    animation_options: Res<'_, AnimationOptions>,
    graphs: Query<'_, '_, &GraphDefinition>,
) {
    if !animation_options.show_animation_skeleton_bones {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for graph in graphs.iter() {
                for n_bone in 0..graph.nodes[graph.current_node_index as usize].clip.poses[graph
                    .nodes[graph.current_node_index as usize]
                    .clip
                    .current_key_frame_index
                    as usize]
                    .skeleton
                    .bone_ids
                    .len()
                {
                    let current_clip = &graph.nodes[graph.current_node_index as usize].clip;
                    let bone_depth: u8 = current_clip.poses
                        [current_clip.current_key_frame_index as usize]
                        .skeleton
                        .get_bone_depth(
                            current_clip.poses[current_clip.current_key_frame_index as usize]
                                .skeleton
                                .bone_ids[n_bone]
                                .unwrap(),
                        )
                        .try_into()
                        .unwrap();
                    let color_interval: u8 = (255
                        / current_clip.poses[current_clip.current_key_frame_index as usize]
                            .skeleton
                            .get_max_bone_depth()
                            .unwrap())
                    .try_into()
                    .unwrap();
                    let debug_color = Color::new(bone_depth * color_interval, 255, 52, 255);
                    builder.add_default_mesh(
                        &graph.nodes[graph.current_node_index as usize].clip.poses[graph.nodes
                            [graph.current_node_index as usize]
                            .clip
                            .current_key_frame_index
                            as usize]
                            .transforms[n_bone]
                            .global,
                        DefaultMeshType::Sphere,
                        debug_color,
                    );
                }
            }
        });
    });
    drop(debug_display);
    drop(bump_allocator_pool);
    drop(animation_options);
    drop(graphs);
}
