use lgn_graphics_api::{
    ColorClearValue, ColorRenderTargetBinding, DeviceContext, LoadOp, ResourceState, StoreOp,
};
use lgn_math::Vec4;
use lgn_transform::components::GlobalTransform;

use crate::{
    cgen::{
        self,
        cgen_type::{PickingData, Transform},
    },
    components::{CameraComponent, LightComponent, ManipulatorComponent, RenderSurface},
    gpu_renderer::{DefaultLayers, GpuInstanceManager, MeshRenderer},
    hl_gfx_api::HLCommandBuffer,
    picking::{ManipulatorManager, PickingManager, PickingState},
    resources::{DefaultMeshType, GpuBufferWithReadback, MeshManager, MeshMetaData},
    RenderContext,
};

pub struct PickingRenderPass {
    count_buffer: GpuBufferWithReadback,
    picked_buffer: GpuBufferWithReadback,
}

impl PickingRenderPass {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            count_buffer: GpuBufferWithReadback::new(device_context, 4),
            picked_buffer: GpuBufferWithReadback::new(
                device_context,
                16 * 1024 * std::mem::size_of::<PickingData>() as u64,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        picking_manager: &PickingManager,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        instance_manager: &GpuInstanceManager,
        manipulator_meshes: &[(&GlobalTransform, &ManipulatorComponent)],
        lights: &[(&LightComponent, &GlobalTransform)],
        mesh_manager: &MeshManager,
        camera: &CameraComponent,
        mesh_renderer: &MeshRenderer,
    ) {
        let device_context = render_context.renderer().device_context();

        let mut count: usize = 0;
        let mut count_readback = self.count_buffer.begin_readback(device_context);
        let mut picked_readback = self.picked_buffer.begin_readback(device_context);

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

        if picking_manager.picking_state() == PickingState::Rendering {
            render_surface
                .hdr_rt_mut()
                .transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

            self.count_buffer.clear_buffer(cmd_buffer);

            cmd_buffer.begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_surface.hdr_rt().rtv(),
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue::default(),
                }],
                &None,
            );

            let pipeline = render_context
                .pipeline_manager()
                .get_pipeline(mesh_renderer.get_tmp_pso_handle(DefaultLayers::Picking as usize))
                .unwrap();

            cmd_buffer.bind_pipeline(pipeline);

            render_context.bind_default_descriptor_sets(cmd_buffer);

            cmd_buffer.bind_index_buffer(
                &render_context
                    .renderer()
                    .static_buffer()
                    .index_buffer_binding(),
            );
            cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

            let mut picking_descriptor_set = cgen::descriptor_set::PickingDescriptorSet::default();
            picking_descriptor_set.set_picked_count(self.count_buffer.rw_view());
            picking_descriptor_set.set_picked_objects(self.picked_buffer.rw_view());
            let picking_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                picking_descriptor_set.descriptor_refs(),
            );
            cmd_buffer.bind_descriptor_set(
                cgen::descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
                picking_descriptor_set_handle,
            );

            let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();
            push_constant_data.set_picking_distance(1.0.into());
            push_constant_data.set_use_gpu_pipeline(1.into());

            cmd_buffer.push_constant(&push_constant_data);

            mesh_renderer.draw(render_context, cmd_buffer, DefaultLayers::Picking as usize);

            for (transform, manipulator) in manipulator_meshes.iter() {
                if manipulator.active {
                    let picking_distance = 50.0;
                    let custom_world = ManipulatorManager::scale_manipulator_for_viewport(
                        transform,
                        &manipulator.local_transform,
                        render_surface,
                        camera,
                    );

                    render_mesh(
                        &custom_world,
                        manipulator.picking_id,
                        picking_distance,
                        mesh_manager.get_default_mesh(manipulator.default_mesh_type),
                        cmd_buffer,
                    );
                }
            }

            for (light, transform) in lights {
                let picking_distance = 1.0;
                let custom_world = transform.with_scale(transform.scale * 0.2);
                render_mesh(
                    &custom_world,
                    light.picking_id,
                    picking_distance,
                    mesh_manager.get_default_mesh(DefaultMeshType::Sphere),
                    cmd_buffer,
                );
            }

            cmd_buffer.end_render_pass();

            self.count_buffer
                .copy_buffer_to_readback(cmd_buffer, &count_readback);
            count_readback.sent_to_gpu(picking_manager.frame_no_for_picking());

            self.picked_buffer
                .copy_buffer_to_readback(cmd_buffer, &picked_readback);
            picked_readback.sent_to_gpu(picking_manager.frame_no_for_picking());
        }

        self.count_buffer.end_readback(count_readback);
        self.picked_buffer.end_readback(picked_readback);
    }
}

fn render_mesh(
    custom_world: &GlobalTransform,
    picking_id: u32,
    picking_distance: f32,
    mesh: &MeshMetaData,
    cmd_buffer: &HLCommandBuffer<'_>,
) {
    let mut push_constant_data = cgen::cgen_type::PickingPushConstantData::default();

    //push_constant_data.set_world((*custom_world).into());
    let mut transform = Transform::default();
    transform.set_translation(custom_world.translation.into());
    transform.set_rotation(Vec4::from(custom_world.rotation).into());
    transform.set_scale(custom_world.scale.into());

    push_constant_data.set_transform(transform);
    push_constant_data.set_mesh_description_offset(mesh.mesh_description_offset.into());
    push_constant_data.set_picking_id(picking_id.into());
    push_constant_data.set_picking_distance(picking_distance.into());
    push_constant_data.set_use_gpu_pipeline(0.into());

    cmd_buffer.push_constant(&push_constant_data);

    if mesh.index_count != 0 {
        cmd_buffer.draw_indexed(mesh.index_count, mesh.index_offset, 0);
    } else {
        cmd_buffer.draw(mesh.vertex_count, 0);
    }
}
