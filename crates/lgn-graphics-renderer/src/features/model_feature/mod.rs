use lgn_data_runtime::Handle;
use lgn_graphics_data::Color;
use lgn_math::Vec4;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen,
    components::VisualComponent,
    core::{
        BinaryWriter, GpuUploadManager, RenderFeature, RenderLayerId, RenderListCallable,
        RenderListSlice, RenderListSliceRequirement, RenderListSliceTyped, RenderObjectId,
        RenderObjectsBuilder, RenderResources, SecondaryTableHandler, TmpDrawContext,
        UploadGPUBuffer, UploadGPUResource, VisibleView,
    },
    gpu_renderer::{MeshRenderer, RenderElement},
    resources::{MeshManager, RenderModel, UnifiedStaticBuffer},
};

mod mesh_instance_manager;
pub(crate) use mesh_instance_manager::*;
pub struct RenderVisual {
    transform: GlobalTransform,
    color: Color,
    color_blend: f32,
    render_model: Handle<RenderModel>,
    picking_id: u32,
}

#[allow(clippy::fallible_impl_from)]
impl From<(&GlobalTransform, &VisualComponent)> for RenderVisual {
    fn from((xform, visual): (&GlobalTransform, &VisualComponent)) -> Self {
        RenderVisual {
            transform: *xform,
            color: visual.color(),
            color_blend: visual.color_blend(),
            render_model: visual.render_model_handle().clone(),
            picking_id: visual.picking_id().unwrap().raw(),
        }
    }
}

pub struct MeshInstanceBlock {
    gpu_instance_ids: Vec<GpuInstanceId>,
    gpu_instance_keys: Vec<MeshInstanceKey>,
}

pub(crate) struct RenderVisualRendererData {
    gpu_instance_block: MeshInstanceBlock,
}

pub(crate) struct RenderVisualRendererDataHandler;

impl SecondaryTableHandler<RenderVisual, RenderVisualRendererData>
    for RenderVisualRendererDataHandler
{
    fn insert(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        render_visual: &RenderVisual,
    ) -> RenderVisualRendererData {
        let mesh_manager = render_resources.get::<MeshManager>();
        let gpu_heap = render_resources.get::<UnifiedStaticBuffer>();
        let gpu_upload_manager = render_resources.get::<GpuUploadManager>();
        let mut mesh_instance_manager = render_resources.get_mut::<MeshInstanceManager>();
        let mut mesh_renderer = render_resources.get_mut::<MeshRenderer>();

        // Transform are updated in their own system
        {
            mesh_instance_manager
                .transform_manager
                .alloc_gpu_data(&render_object_id);
        }
        // Color are updated in the update function
        {
            mesh_instance_manager
                .color_manager
                .alloc_gpu_data(&render_object_id);
        }
        // Picking is allocated and updated at creation time
        {
            mesh_instance_manager
                .picking_data_manager
                .alloc_gpu_data(&render_object_id);
            let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
            picking_data.set_picking_id(render_visual.picking_id.into());
            mesh_instance_manager
                .picking_data_manager
                .sync_update_gpu_data(&render_object_id, &picking_data);
        }

        let render_model_handle = &render_visual.render_model;
        let render_model_guard = render_model_handle.get().unwrap();
        let render_model = &*render_model_guard;

        //
        // Gpu instances
        //

        let mut gpu_instance_ids = Vec::new();
        let mut gpu_instance_keys = Vec::new();
        let mesh_reader = mesh_manager.read();
        for (mesh_index, mesh) in render_model.mesh_instances().iter().enumerate() {
            //
            // Mesh
            //
            let render_mesh = mesh_reader.get_render_mesh(mesh.mesh_id);

            //
            // Gpu instance
            //

            let instance_vas = GpuInstanceVas {
                submesh_va: render_mesh.mesh_description_offset,
                material_va: mesh.material_va as u32,
                color_va: mesh_instance_manager
                    .color_manager
                    .gpuheap_addr_for_key(&render_object_id) as u32,
                transform_va: mesh_instance_manager
                    .transform_manager
                    .gpuheap_addr_for_key(&render_object_id) as u32,
                picking_data_va: mesh_instance_manager
                    .picking_data_manager
                    .gpuheap_addr_for_key(&render_object_id)
                    as u32,
            };

            let gpu_instance_key = MeshInstanceKey {
                render_object_id,
                mesh_index,
            };
            gpu_instance_keys.push(gpu_instance_key);

            let gpu_data_allocation = mesh_instance_manager
                .va_table_manager
                .alloc_gpu_data(&gpu_instance_key);

            let gpu_instance_id = GpuInstanceId(gpu_data_allocation.index());
            gpu_instance_ids.push(gpu_instance_id);

            mesh_instance_manager
                .va_table_adresses
                .set_va_table_address_for_gpu_instance(&gpu_upload_manager, gpu_data_allocation);

            let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
            gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
            gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());
            gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
            gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
            gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

            let mut binary_writer = BinaryWriter::new();
            binary_writer.write(&gpu_instance_va_table);

            gpu_upload_manager.push(UploadGPUResource::Buffer(UploadGPUBuffer {
                src_data: binary_writer.take(),
                dst_buffer: gpu_heap.buffer().clone(),
                dst_offset: gpu_data_allocation.gpuheap_addr(),
            }));

            mesh_renderer.register_material(mesh.material_id);
            mesh_renderer.register_element(&RenderElement::new(
                gpu_instance_id,
                mesh.material_id,
                render_mesh,
            ));
        }

        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color((u32::from(render_visual.color)).into());
        instance_color.set_color_blend(render_visual.color_blend.into());
        mesh_instance_manager
            .color_manager
            .sync_update_gpu_data(&render_object_id, &instance_color);

        let mut world = cgen::cgen_type::TransformData::default();
        world.set_translation(render_visual.transform.translation.into());
        world.set_rotation(Vec4::from(render_visual.transform.rotation).into());
        world.set_scale(render_visual.transform.scale.into());
        mesh_instance_manager
            .transform_manager
            .sync_update_gpu_data(&render_object_id, &world);

        RenderVisualRendererData {
            gpu_instance_block: MeshInstanceBlock {
                gpu_instance_ids,
                gpu_instance_keys,
            },
        }
    }

    fn update(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        render_visual: &RenderVisual,
        _render_visual_private_data: &mut RenderVisualRendererData,
    ) {
        let mesh_instance_manager = render_resources.get_mut::<MeshInstanceManager>();

        //
        // TODO(vdbdd): RenderModel (or one of its dependencies has changed)
        //

        //
        // update instance color
        //
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color((u32::from(render_visual.color)).into());
        instance_color.set_color_blend(render_visual.color_blend.into());
        mesh_instance_manager
            .color_manager
            .sync_update_gpu_data(&render_object_id, &instance_color);

        //
        // update transform
        //
        let mut world = cgen::cgen_type::TransformData::default();
        world.set_translation(render_visual.transform.translation.into());
        world.set_rotation(Vec4::from(render_visual.transform.rotation).into());
        world.set_scale(render_visual.transform.scale.into());
        mesh_instance_manager
            .transform_manager
            .sync_update_gpu_data(&render_object_id, &world);
    }

    fn remove(
        &self,
        render_resources: &RenderResources,
        render_object_id: RenderObjectId,
        _render_visual: &RenderVisual,
        render_visual_private_data: &mut RenderVisualRendererData,
    ) {
        let mut mesh_instance_manager = render_resources.get_mut::<MeshInstanceManager>();
        let mut mesh_renderer = render_resources.get_mut::<MeshRenderer>();

        for gpu_instance_id in render_visual_private_data
            .gpu_instance_block
            .gpu_instance_ids
            .drain(..)
        {
            mesh_renderer.unregister_element(gpu_instance_id);
        }

        for gpu_instance_key in render_visual_private_data
            .gpu_instance_block
            .gpu_instance_keys
            .drain(..)
        {
            mesh_instance_manager
                .va_table_manager
                .remove_gpu_data(&gpu_instance_key);
        }

        mesh_instance_manager
            .transform_manager
            .remove_gpu_data(&render_object_id);
        mesh_instance_manager
            .color_manager
            .remove_gpu_data(&render_object_id);
        mesh_instance_manager
            .picking_data_manager
            .remove_gpu_data(&render_object_id);
    }
}

impl RenderVisualRendererDataHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[allow(dead_code)]
struct TmpKickLayer {
    render_layer_id: RenderLayerId,
}

#[cfg(debug_assertions)]
impl Drop for TmpKickLayer {
    fn drop(&mut self) {
        // println!("TmpKickLayer dropped");
    }
}

impl RenderListCallable for TmpKickLayer {
    fn call(&self, _draw_context: &mut TmpDrawContext) {
        #[cfg(debug_assertions)]
        {
            // println!("TmpKickLayer called: {}", self.render_layer_id);
        }
    }
}

pub struct ModelFeature {
    // mesh_instance_manager: MeshInstanceManager,
}

impl ModelFeature {
    pub fn new(render_objects: &mut RenderObjectsBuilder) -> Self {
        render_objects
            .add_primary_table::<RenderVisual>()
            .add_secondary_table_with_handler::<RenderVisual, RenderVisualRendererData>(Box::new(
                RenderVisualRendererDataHandler::new(),
            ));

        Self {}
    }
}

impl RenderFeature for ModelFeature {
    fn get_render_list_requirement(
        &self,
        _view_id: &VisibleView,
        _layer_id: RenderLayerId,
    ) -> Option<RenderListSliceRequirement> {
        Some(RenderListSliceRequirement::new::<TmpKickLayer>(1))
    }

    fn prepare_render_list(
        &self,
        _view_id: &VisibleView,
        render_layer_id: RenderLayerId,
        render_list_slice: RenderListSlice,
    ) {
        let render_list_slice = RenderListSliceTyped::<TmpKickLayer>::new(render_list_slice);

        for writer in render_list_slice.iter() {
            writer.write(0, TmpKickLayer { render_layer_id });
        }
    }
}
