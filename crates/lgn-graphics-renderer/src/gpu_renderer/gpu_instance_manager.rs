use lgn_app::{App, EventWriter};
use lgn_ecs::prelude::{Changed, Entity, Or, Query, Res, ResMut, Without};
use lgn_graphics_api::{BufferView, VertexBufferBinding};
use lgn_math::Vec4;
use lgn_transform::components::Parent;

use crate::{
    cgen,
    components::{ManipulatorComponent, MaterialComponent, VisualComponent},
    labels::RenderStage,
    picking::{PickingIdContext, PickingManager},
    resources::{
        GpuDataManager, GpuEntityColorManager, GpuEntityTransformManager, GpuPickingDataManager,
        GpuVaTableForGpuInstance, MaterialManager, MeshManager, MissingVisualTracker, ModelManager,
        UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
    },
    Renderer,
};

use super::{GpuInstanceEvent, RenderElement};

pub(crate) type GpuVaTableManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceVATable>;

pub(crate) struct GpuInstanceVas {
    pub submesh_va: u32,
    pub material_va: u32,

    pub color_va: u32,
    pub transform_va: u32,
    pub picking_data_va: u32,
}

pub(crate) struct GpuInstanceManager {
    va_table_manager: GpuVaTableManager,
    va_table_adresses: GpuVaTableForGpuInstance,
}

impl GpuInstanceManager {
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            va_table_manager: GpuVaTableManager::new(64 * 1024, 4096),
            va_table_adresses: GpuVaTableForGpuInstance::new(allocator),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(RenderStage::Prepare, update_gpu_instances);
    }

    pub fn add_gpu_instance(
        &mut self,
        entity: Entity,
        allocator: &UnifiedStaticBufferAllocator,
        updater: &mut UniformGPUDataUpdater,
        instance_vas: &GpuInstanceVas,
    ) -> u32 {
        let (gpu_instance_id, va_table_address) =
            self.va_table_manager.alloc_gpu_data(entity, allocator);

        self.va_table_adresses
            .set_va_table_address_for_gpu_instance(
                updater,
                gpu_instance_id,
                va_table_address as u32,
            );

        let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
        gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
        gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());

        // Fallback to default material if we do not have a specific material set
        if instance_vas.material_va == u32::MAX {
            // gpu_instance_va_table
            //     .set_material_data_va(uniform_data.default_material_gpu_offset.into());
        } else {
            gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
        }
        gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
        gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

        updater.add_update_jobs(&[gpu_instance_va_table], va_table_address);

        gpu_instance_id
    }

    pub fn remove_gpu_instance(&mut self, entity: Entity) -> Option<Vec<u32>> {
        self.va_table_manager.remove_gpu_data(&entity)
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.va_table_adresses.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.va_table_adresses
            .structured_buffer_view(struct_size, read_only)
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn update_gpu_instances(
    renderer: Res<'_, Renderer>,
    picking_manager: Res<'_, PickingManager>,
    mut picking_data_manager: ResMut<'_, GpuPickingDataManager>,
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    material_manager: Res<'_, MaterialManager>,
    color_manager: Res<'_, GpuEntityColorManager>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
    mut event_writer: EventWriter<'_, '_, GpuInstanceEvent>,
    instance_query: Query<
        '_,
        '_,
        (Entity, &VisualComponent, Option<&MaterialComponent>),
        (
            Or<(Changed<VisualComponent>, Changed<Parent>)>,
            Without<ManipulatorComponent>,
        ),
    >,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut picking_context = PickingIdContext::new(&picking_manager);

    for (entity, _, _) in instance_query.iter() {
        picking_data_manager.remove_gpu_data(&entity);
        if let Some(removed_ids) = instance_manager.remove_gpu_instance(entity) {
            event_writer.send(GpuInstanceEvent::Removed(removed_ids));
        }
    }

    for (entity, visual, mat_component) in instance_query.iter() {
        let color: (f32, f32, f32, f32) = (
            f32::from(visual.color.r) / 255.0f32,
            f32::from(visual.color.g) / 255.0f32,
            f32::from(visual.color.b) / 255.0f32,
            f32::from(visual.color.a) / 255.0f32,
        );
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());

        instance_color.set_color_blend(visual.color_blend.into());

        color_manager.update_gpu_data(&entity, 0, &instance_color, &mut updater);

        let mut material_key = None;
        if let Some(material) = mat_component {
            material_key = Some(material.material_id);
        }

        picking_data_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());

        let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
        picking_data.set_picking_id(picking_context.acquire_picking_id(entity).into());
        picking_data_manager.update_gpu_data(&entity, 0, &picking_data, &mut updater);

        let (model_meta_data, ready) = model_manager.get_model_meta_data(visual);
        if !ready {
            if let Some(model_resource_id) = &visual.model_resource_id {
                missing_visuals_tracker.add_entity(*model_resource_id, entity);
            }
        }

        let mut added_instances = Vec::with_capacity(model_meta_data.meshes.len());
        for mesh in &model_meta_data.meshes {
            let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);

            let instance_vas = GpuInstanceVas {
                submesh_va: mesh_meta_data.mesh_description_offset,
                material_va: if mesh.material_id != u32::MAX {
                    mesh.material_id
                } else {
                    material_manager.gpu_data().va_for_index(material_key, 0) as u32
                },
                color_va: color_manager.va_for_index(Some(entity), 0) as u32,
                transform_va: transform_manager.va_for_index(Some(entity), 0) as u32,
                picking_data_va: picking_data_manager.va_for_index(Some(entity), 0) as u32,
            };

            let gpu_instance_id = instance_manager.add_gpu_instance(
                entity,
                renderer.static_buffer_allocator(),
                &mut updater,
                &instance_vas,
            );

            added_instances.push((
                if mesh.material_id != u32::MAX {
                    mesh.material_index
                } else {
                    material_manager.gpu_data().id_for_index(material_key, 0)
                },
                RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
            ));
        }
        event_writer.send(GpuInstanceEvent::Added(added_instances));
    }

    renderer.add_update_job_block(updater.job_blocks());
}
