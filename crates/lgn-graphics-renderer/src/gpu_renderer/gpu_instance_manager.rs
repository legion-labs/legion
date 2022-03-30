use lgn_app::{App, EventWriter};
use lgn_ecs::{
    prelude::{Changed, Entity, Or, Query, RemovedComponents, Res, ResMut},
    schedule::{SystemLabel, SystemSet},
};
use lgn_graphics_api::{BufferView, VertexBufferBinding};
use lgn_hierarchy::prelude::Parent;
use lgn_math::Vec4;
use lgn_tasks::ComputeTaskPool;
use lgn_tracing::warn;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen,
    components::VisualComponent,
    labels::RenderStage,
    picking::{PickingIdContext, PickingManager},
    resources::{
        GpuDataManager, GpuVaTableForGpuInstance, MaterialManager, MeshManager,
        MissingVisualTracker, ModelManager, UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
    },
    Renderer,
};

use super::{GpuInstanceEvent, RenderElement};

type GpuPickingDataManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstancePickingData>;
type GpuEntityColorManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceColor>;
type GpuEntityTransformManager = GpuDataManager<Entity, cgen::cgen_type::Transform>;
pub(crate) type GpuVaTableManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceVATable>;

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum GpuInstanceManagerLabel {
    UpdateDone,
}

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
        app.insert_resource(GpuEntityTransformManager::new(64 * 1024, 1024));
        app.insert_resource(GpuEntityColorManager::new(64 * 1024, 256));
        app.insert_resource(GpuPickingDataManager::new(64 * 1024, 1024));

        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(update_gpu_instances)
                .with_system(remove_gpu_instances)
                .label(GpuInstanceManagerLabel::UpdateDone),
        );
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(upload_transform_data)
                .after(GpuInstanceManagerLabel::UpdateDone),
        );
    }

    fn add_gpu_instance(
        &mut self,
        entity: Entity,
        allocator: &UnifiedStaticBufferAllocator,
        updater: &mut UniformGPUDataUpdater,
        instance_vas: &GpuInstanceVas,
    ) -> u32 {
        let (gpu_instance_id, va_table_address) =
            self.va_table_manager.alloc_gpu_data(&entity, allocator);

        self.va_table_adresses
            .set_va_table_address_for_gpu_instance(
                updater,
                gpu_instance_id,
                va_table_address as u32,
            );

        let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
        gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
        gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());
        gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
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
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    material_manager: Res<'_, MaterialManager>,
    mut transform_manager: ResMut<'_, GpuEntityTransformManager>,
    mut color_manager: ResMut<'_, GpuEntityColorManager>,
    mut picking_data_manager: ResMut<'_, GpuPickingDataManager>,
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    mut event_writer: EventWriter<'_, '_, GpuInstanceEvent>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
    instance_query: Query<
        '_,
        '_,
        (Entity, &GlobalTransform, &VisualComponent),
        Or<(Changed<VisualComponent>, Changed<Parent>)>,
    >,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut picking_context = PickingIdContext::new(&picking_manager);

    // First remove any registered data
    for (entity, _, _) in instance_query.iter() {
        transform_manager.remove_gpu_data(&entity);
        color_manager.remove_gpu_data(&entity);
        picking_data_manager.remove_gpu_data(&entity);
        if let Some(removed_ids) = instance_manager.remove_gpu_instance(entity) {
            event_writer.send(GpuInstanceEvent::Removed(removed_ids));
        }
    }

    for (entity, transform, visual) in instance_query.iter() {
        //
        // Transform
        //
        let mut world = cgen::cgen_type::Transform::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        transform_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
        transform_manager.update_gpu_data(&entity, 0, &world, &mut updater);

        //
        // Color
        //
        let color: (f32, f32, f32, f32) = (
            f32::from(visual.color.r) / 255.0f32,
            f32::from(visual.color.g) / 255.0f32,
            f32::from(visual.color.b) / 255.0f32,
            f32::from(visual.color.a) / 255.0f32,
        );
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());
        instance_color.set_color_blend(visual.color_blend.into());

        color_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
        color_manager.update_gpu_data(&entity, 0, &instance_color, &mut updater);

        //
        // Picking part
        //
        let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
        picking_data.set_picking_id(picking_context.acquire_picking_id(entity).into());

        picking_data_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
        picking_data_manager.update_gpu_data(&entity, 0, &picking_data, &mut updater);

        //
        // Model (might no be ready. it returns a default model)
        // TODO(vdbdd): should be managed at call site (default model depending on some criterias)
        //
        let (model_meta_data, ready) =
            model_manager.get_model_meta_data(visual.model_resource_id.as_ref());
        if !ready {
            warn!(
                "Dependency issue. Model {} not loaded for entity {:?}",
                visual.model_resource_id.unwrap(),
                entity
            );
            if let Some(model_resource_id) = &visual.model_resource_id {
                missing_visuals_tracker.add_resource_entity_dependency(*model_resource_id, entity);
            }
        }

        //
        // Gpu instances
        //
        let mut added_instances = Vec::with_capacity(model_meta_data.meshes.len());
        let default_material_id = material_manager.get_default_material_id();

        for mesh in &model_meta_data.meshes {
            //
            // Mesh
            //
            let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);

            //
            // Material (might not be valid)
            //
            let material_id = if material_manager.is_material_ready(mesh.material_id) {
                mesh.material_id
            } else {
                let material_resource_id = material_manager
                    .get_material(mesh.material_id)
                    .resource_id();
                warn!(
                    "Dependency issue. Material {} not ready for entity {:?}",
                    material_resource_id, entity
                );
                missing_visuals_tracker
                    .add_resource_entity_dependency(*material_resource_id, entity);
                default_material_id
            };

            //
            // Gpu instance
            //

            let instance_vas = GpuInstanceVas {
                submesh_va: mesh_meta_data.mesh_description_offset,
                material_va: material_manager.get_material(material_id).va() as u32,
                color_va: color_manager.va_for_index(&entity, 0) as u32,
                transform_va: transform_manager.va_for_index(&entity, 0) as u32,
                picking_data_va: picking_data_manager.va_for_index(&entity, 0) as u32,
            };

            let gpu_instance_id = instance_manager.add_gpu_instance(
                entity,
                renderer.static_buffer_allocator(),
                &mut updater,
                &instance_vas,
            );

            added_instances.push((
                material_id,
                RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
            ));
        }
        event_writer.send(GpuInstanceEvent::Added(added_instances));
    }

    renderer.add_update_job_block(updater.job_blocks());
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn upload_transform_data(
    task_pool: Res<'_, ComputeTaskPool>,
    renderer: Res<'_, Renderer>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
    query: Query<'_, '_, (Entity, &GlobalTransform, &VisualComponent), Changed<GlobalTransform>>,
) {
    query.par_for_each(&task_pool, 256, |(entity, transform, _)| {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        let mut world = cgen::cgen_type::Transform::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        transform_manager.update_gpu_data(&entity, 0, &world, &mut updater);

        renderer.add_update_job_block(updater.job_blocks());
    });
}

#[allow(clippy::needless_pass_by_value)]
fn remove_gpu_instances(
    mut transform_manager: ResMut<'_, GpuEntityTransformManager>,
    mut color_manager: ResMut<'_, GpuEntityColorManager>,
    mut picking_data_manager: ResMut<'_, GpuPickingDataManager>,
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    mut event_writer: EventWriter<'_, '_, GpuInstanceEvent>,
    removed_visual_components: RemovedComponents<'_, VisualComponent>,
) {
    for entity in removed_visual_components.iter() {
        transform_manager.remove_gpu_data(&entity);
        color_manager.remove_gpu_data(&entity);
        picking_data_manager.remove_gpu_data(&entity);
        if let Some(removed_ids) = instance_manager.remove_gpu_instance(entity) {
            event_writer.send(GpuInstanceEvent::Removed(removed_ids));
        }
    }
}
