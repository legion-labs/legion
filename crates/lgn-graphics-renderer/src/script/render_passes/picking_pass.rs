use lgn_graphics_api::{ColorClearValue, CommandBuffer};
use lgn_math::Vec4;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen::{
        self,
        cgen_type::{PickingData, TransformData},
    },
    core::{
        RenderGraphBuilder, RenderGraphLoadState, RenderGraphResourceId, RenderGraphViewId,
        RenderObjectQuery, RenderObjects, RENDER_LAYER_PICKING,
    },
    gpu_renderer::{GpuInstanceManager, MeshRenderer},
    lighting::RenderLight,
    picking::{ManipulatorManager, PickingState},
    resources::{DefaultMeshType, MeshManager, RenderMesh},
    RenderScope,
};

pub struct PickingPass;

impl PickingPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        radiance_write_rt_view_id: RenderGraphViewId,
        draw_count_buffer_id: RenderGraphResourceId,
        draw_args_buffer_id: RenderGraphResourceId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_scope("Picking", |builder| {
            builder
                .add_compute_pass("Picking begin readback", |compute_pass_builder| {
                    compute_pass_builder.execute(|_, execute_context, cmd_buffer| {
                        let render_context = &execute_context.render_context;
                        let mut picking_renderpass = execute_context
                            .debug_stuff
                            .picking_renderpass.write();
                        let picking_manager = render_context.picking_manager;
                        let render_scope =
                            execute_context.render_resources.get::<RenderScope>();

                        let frame_index = render_scope.frame_idx() as usize;
                        let mut count: usize = 0;
                        let mut count_readback = picking_renderpass
                            .count_buffer
                            .begin_readback(frame_index, render_context.device_context);
                        let mut picked_readback = picking_renderpass
                            .picked_buffer
                            .begin_readback(frame_index, render_context.device_context);

                        count_readback.read_gpu_data(
                            0,
                            usize::MAX,
                            picking_manager.frame_no_picked(),
                            |data: &[u32]| {
                                count = data[0] as usize;
                            },
                        );

                        picked_readback.read_gpu_data(
                            0,
                            count,
                            picking_manager.frame_no_picked(),
                            |data: &[PickingData]| {
                                picking_manager.set_picked(data);
                            },
                        );

                        execute_context.count_readback = count_readback.transfer();
                        execute_context.picked_readback = picked_readback.transfer();

                        if picking_manager.picking_state() == PickingState::Rendering {
                            picking_renderpass.count_buffer.clear_buffer(cmd_buffer);
                        }
                    })
                })
                .add_graphics_pass("PickingDraw", |graphics_pass_builder| {
                    graphics_pass_builder
                        .render_target(
                            0,
                            radiance_write_rt_view_id,
                            RenderGraphLoadState::ClearColor(ColorClearValue::default()),
                        )
                        .execute(move |context, execute_context, cmd_buffer| {
                            let render_context = &execute_context.render_context;
                            let picking_renderpass = execute_context
                                .debug_stuff
                                .picking_renderpass.write();
                            let picking_manager = render_context.picking_manager;

                            if picking_manager.picking_state() == PickingState::Rendering {
                                let mesh_renderer =
                                    execute_context.render_resources.get::<MeshRenderer>();
                                let instance_manager =
                                    execute_context.render_resources.get::<GpuInstanceManager>();
                                let mesh_manager =
                                    execute_context.render_resources.get::<MeshManager>();

                                let pipeline = render_context
                                    .pipeline_manager
                                    .get_pipeline(
                                        mesh_renderer
                                            .get_tmp_pso_handle(RENDER_LAYER_PICKING.index()),
                                    )
                                    .unwrap();

                                cmd_buffer.cmd_bind_pipeline(pipeline);

                                render_context.bind_default_descriptor_sets(cmd_buffer);

                                cmd_buffer.cmd_bind_index_buffer(
                                    render_context.static_buffer.index_buffer_binding(),
                                );
                                cmd_buffer.cmd_bind_vertex_buffer(
                                    0,
                                    instance_manager.vertex_buffer_binding(),
                                );

                                let mut picking_descriptor_set =
                                    cgen::descriptor_set::PickingDescriptorSet::default();
                                picking_descriptor_set
                                    .set_picked_count(picking_renderpass.count_buffer.rw_view());
                                picking_descriptor_set
                                    .set_picked_objects(picking_renderpass.picked_buffer.rw_view());
                                let picking_descriptor_set_handle = render_context
                                    .write_descriptor_set(
                                        cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                                        picking_descriptor_set.descriptor_refs(),
                                    );
                                cmd_buffer.cmd_bind_descriptor_set_handle(
                                    cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                                    picking_descriptor_set_handle,
                                );

                                let mut push_constant_data =
                                    cgen::cgen_type::PickingPushConstantData::default();
                                push_constant_data.set_picking_distance(1.0.into());
                                push_constant_data.set_use_gpu_pipeline(1.into());

                                cmd_buffer.cmd_push_constant_typed(&push_constant_data);

                                let render_context = &execute_context.render_context;
                                let mesh_renderer =
                                    execute_context.render_resources.get::<MeshRenderer>();

                                mesh_renderer.render_layer_batches[RENDER_LAYER_PICKING.index()].draw(
                                    render_context,
                                    cmd_buffer,
                                    Some(context.get_buffer(draw_args_buffer_id)),
                                    Some(context.get_buffer(draw_count_buffer_id)),
                                );

                                let manipulator_meshes =
                                render_context.manipulator_drawables;
                                let render_viewport = execute_context.debug_stuff.render_viewport;
                                let render_camera = execute_context.debug_stuff.render_camera;

                                let view_transform = render_camera.view_transform();
                                let projection = render_camera.build_projection(render_viewport.extents().width as f32, render_viewport.extents().height as f32);
                                let mesh_reader = mesh_manager.read();
                                for (transform, manipulator) in manipulator_meshes.iter() {
                                    if manipulator.active {
                                        let picking_distance = 50.0;
                                        let custom_world =
                                            ManipulatorManager::scale_manipulator_for_viewport(
                                                transform,
                                                &manipulator.local_transform,
                                                projection,
                                                &view_transform,
                                            );

                                        Self::render_mesh(
                                            &custom_world,
                                            manipulator.picking_id,
                                            picking_distance,
                                            mesh_reader
                                                .get_default_mesh(manipulator.default_mesh_type),
                                            cmd_buffer,
                                        );
                                    }
                                }

                                let render_objects =
                                    execute_context.render_resources.get::<RenderObjects>();
                                let render_lights =
                                    RenderObjectQuery::<RenderLight>::new(&render_objects);

                                for render_light in render_lights.iter() {
                                    let picking_distance = 1.0;
                                    let custom_world = render_light
                                        .transform
                                        .with_scale(render_light.transform.scale * 0.2);
                                    Self::render_mesh(
                                        &custom_world,
                                        render_light.picking_id,
                                        picking_distance,
                                        mesh_reader.get_default_mesh(DefaultMeshType::Sphere),
                                        cmd_buffer,
                                    );
                                }
                            }
                        })
                })
                .add_compute_pass("Picking end readback", |compute_pass_builder| {
                    compute_pass_builder.execute(|_, execute_context, cmd_buffer| {
                        let render_context = &execute_context.render_context;
                        let mut picking_renderpass = execute_context
                            .debug_stuff
                            .picking_renderpass.write();
                        let picking_manager = render_context.picking_manager;
                        let render_scope =
                            execute_context.render_resources.get::<RenderScope>();

                        let mut count_readback = execute_context.count_readback.transfer();
                        let mut picked_readback = execute_context.picked_readback.transfer();

                        if picking_manager.picking_state() == PickingState::Rendering {
                            picking_renderpass
                                .count_buffer
                                .copy_buffer_to_readback(cmd_buffer, &count_readback);
                            count_readback.sent_to_gpu(picking_manager.frame_no_for_picking());

                            picking_renderpass
                                .picked_buffer
                                .copy_buffer_to_readback(cmd_buffer, &picked_readback);
                            picked_readback.sent_to_gpu(picking_manager.frame_no_for_picking());
                        }

                        let frame_index = render_scope.frame_idx() as usize;
                        picking_renderpass.count_buffer.end_readback(frame_index, count_readback);
                        picking_renderpass
                            .picked_buffer
                            .end_readback(frame_index, picked_readback);
                    })
                })
        })
    }

    fn render_mesh(
        custom_world: &GlobalTransform,
        picking_id: u32,
        picking_distance: f32,
        mesh: &RenderMesh,
        cmd_buffer: &mut CommandBuffer,
    ) {
        let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();

        //push_constant_data.set_world((*custom_world).into());
        let mut transform = TransformData::default();
        transform.set_translation(custom_world.translation.into());
        transform.set_rotation(Vec4::from(custom_world.rotation).into());
        transform.set_scale(custom_world.scale.into());

        push_constant_data.set_transform(transform);
        push_constant_data.set_mesh_description_offset(mesh.mesh_description_offset.into());
        push_constant_data.set_picking_id(picking_id.into());
        push_constant_data.set_picking_distance(picking_distance.into());
        push_constant_data.set_use_gpu_pipeline(0.into());

        cmd_buffer.cmd_push_constant_typed(&push_constant_data);

        cmd_buffer.cmd_draw_indexed(mesh.index_count.get(), mesh.index_offset, 0);
    }
}
