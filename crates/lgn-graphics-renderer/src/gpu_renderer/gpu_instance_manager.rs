use std::collections::BTreeMap;

use lgn_app::App;
use lgn_ecs::{
    prelude::{Changed, Entity, Query, RemovedComponents, Res, ResMut},
    schedule::{SystemLabel, SystemSet},
};
use lgn_graphics_api::{BufferView, VertexBufferBinding};

use lgn_math::Vec4;
use lgn_tracing::warn;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen,
    components::VisualComponent,
    labels::RenderStage,
    picking::{PickingIdContext, PickingManager},
    resources::{
        GpuDataAllocation, GpuDataManager, MaterialManager, MeshManager, MissingVisualTracker,
        ModelManager, StaticBufferAllocation, UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
    },
    Renderer,
};

use super::RenderElement;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GpuInstanceKey {
    entity: Entity,
    mesh_index: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuInstanceId(u32);

impl From<u32> for GpuInstanceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl GpuInstanceId {
    pub fn index(self) -> u32 {
        self.0
    }
}

type GpuPickingDataManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstancePickingData>;
type GpuEntityColorManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceColor>;
type GpuEntityTransformManager = GpuDataManager<Entity, cgen::cgen_type::Transform>;
type GpuVaTableManager = GpuDataManager<GpuInstanceKey, cgen::cgen_type::GpuInstanceVATable>;

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GpuInstanceManagerLabel {
    UpdateDone,
}

struct GpuInstanceVas {
    pub submesh_va: u32,
    pub material_va: u32,
    pub color_va: u32,
    pub transform_va: u32,
    pub picking_data_va: u32,
}

struct GpuInstanceBlock {
    // entity: Entity,
    gpu_instance_ids: Vec<GpuInstanceId>,
    gpu_instance_keys: Vec<GpuInstanceKey>,
}

struct GpuVaTableForGpuInstance {
    static_allocation: StaticBufferAllocation,
}

impl GpuVaTableForGpuInstance {
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            static_allocation: allocator.allocate_segment(4 * 1024 * 1024),
        }
    }

    pub fn set_va_table_address_for_gpu_instance(
        &self,
        updater: &mut UniformGPUDataUpdater,
        gpu_data_allocation: GpuDataAllocation, // gpu_instance_id: GpuInstanceId,
                                                // va_table_address: u64
    ) {
        let offset_for_gpu_instance =
            self.static_allocation.offset() + u64::from(gpu_data_allocation.index()) * 4;

        updater.add_update_jobs(
            std::slice::from_ref(&u32::try_from(gpu_data_allocation.va_address()).unwrap()),
            offset_for_gpu_instance,
        );
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.static_allocation.vertex_buffer_binding()
    }

    pub fn create_structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.static_allocation
            .create_structured_buffer_view(struct_size, read_only)
    }
}

pub(crate) struct GpuInstanceManager {
    transform_manager: GpuEntityTransformManager,
    color_manager: GpuEntityColorManager,
    picking_data_manager: GpuPickingDataManager,
    va_table_manager: GpuVaTableManager,
    va_table_adresses: GpuVaTableForGpuInstance,
    entity_to_gpu_instance_block: BTreeMap<Entity, GpuInstanceBlock>,
    added_render_elements: Vec<RenderElement>,
    removed_gpu_instance_ids: Vec<GpuInstanceId>,
}

impl GpuInstanceManager {
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            // TODO(vdbdd): as soon as we have a stable ID, we can move the transform in their own manager.
            transform_manager: GpuEntityTransformManager::new(1024),
            color_manager: GpuEntityColorManager::new(256),
            picking_data_manager: GpuPickingDataManager::new(1024),
            va_table_manager: GpuVaTableManager::new(4096),
            va_table_adresses: GpuVaTableForGpuInstance::new(allocator),
            entity_to_gpu_instance_block: BTreeMap::new(),
            added_render_elements: Vec::new(),
            removed_gpu_instance_ids: Vec::new(),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(update_gpu_instances)
                .label(GpuInstanceManagerLabel::UpdateDone),
        );
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(upload_transform_data)
                .after(GpuInstanceManagerLabel::UpdateDone),
        );
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.va_table_adresses.vertex_buffer_binding()
    }

    pub fn create_structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.va_table_adresses
            .create_structured_buffer_view(struct_size, read_only)
    }

    // fn add_gpu_instance(
    //     &mut self,
    //     entity: Entity,
    //     mesh_id: u32,
    //     allocator: &UnifiedStaticBufferAllocator,
    //     updater: &mut UniformGPUDataUpdater,
    //     instance_vas: &GpuInstanceVas,
    // ) -> (GpuInstanceKey, GpuInstanceId) {
    //     let gpu_instance_key = GpuInstanceKey { entity, mesh_index };

    //     let gpu_data_allocation = self
    //         .va_table_manager
    //         .alloc_gpu_data(&gpu_instance_key, allocator);

    //     let gpu_instance_id = gpu_data_allocation.index().into();

    //     self.va_table_adresses
    //         .set_va_table_address_for_gpu_instance(
    //             updater,
    //             gpu_instance_id,
    //             gpu_data_allocation.va_address(),
    //         );

    //     let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
    //     gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
    //     gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());
    //     gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
    //     gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
    //     gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

    //     updater.add_update_jobs(&[gpu_instance_va_table], gpu_data_allocation.va_address());

    //     (gpu_instance_key, gpu_instance_id)
    // }

    fn clear_transient_containers(&mut self) {
        self.added_render_elements.clear();
        self.removed_gpu_instance_ids.clear();
    }

    pub fn for_each_render_element_added(&self, func: impl FnMut(&RenderElement)) {
        self.added_render_elements.iter().for_each(func);
    }

    pub fn for_each_removed_gpu_instance_id(&self, func: impl FnMut(&GpuInstanceId)) {
        self.removed_gpu_instance_ids.iter().for_each(func);
    }
    #[allow(clippy::too_many_arguments)]
    fn add_gpu_instance_block(
        &mut self,
        entity: Entity,
        visual: &VisualComponent,
        renderer: &Renderer,
        model_manager: &ModelManager,
        mesh_manager: &MeshManager,
        material_manager: &MaterialManager,
        missing_visuals_tracker: &mut MissingVisualTracker,
        updater: &mut UniformGPUDataUpdater,
    ) {
        assert!(!self.entity_to_gpu_instance_block.contains_key(&entity));

        self.transform_manager
            .alloc_gpu_data(&entity, renderer.static_buffer_allocator());
        self.color_manager
            .alloc_gpu_data(&entity, renderer.static_buffer_allocator());
        self.picking_data_manager
            .alloc_gpu_data(&entity, renderer.static_buffer_allocator());

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

        // let mut added_instances = Vec::with_capacity(model_meta_data.meshes.len());
        let mut gpu_instance_ids = Vec::new();
        let mut gpu_instance_keys = Vec::new();
        let default_material_id = material_manager.get_default_material_id();

        for (mesh_index, mesh) in model_meta_data.meshes.iter().enumerate() {
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
                color_va: self.color_manager.va_for_key(&entity) as u32,
                transform_va: self.transform_manager.va_for_key(&entity) as u32,
                picking_data_va: self.picking_data_manager.va_for_key(&entity) as u32,
            };

            let gpu_instance_key = GpuInstanceKey { entity, mesh_index };

            let gpu_data_allocation = self
                .va_table_manager
                .alloc_gpu_data(&gpu_instance_key, renderer.static_buffer_allocator());

            let gpu_instance_id = gpu_data_allocation.index().into();

            self.va_table_adresses
                .set_va_table_address_for_gpu_instance(
                    updater,
                    gpu_data_allocation
                    // gpu_instance_id,
                    // gpu_data_allocation.va_address(),
                );

            let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
            gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
            gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());
            gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
            gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
            gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

            updater.add_update_jobs(&[gpu_instance_va_table], gpu_data_allocation.va_address());

            // let (gpu_instance_key, gpu_instance_id) = self.add_gpu_instance(
            //     entity,
            //     mesh.mesh_id,
            //     renderer.static_buffer_allocator(),
            //     &mut updater,
            //     &instance_vas,
            // );

            gpu_instance_keys.push(gpu_instance_key);
            gpu_instance_ids.push(gpu_instance_id);

            self.added_render_elements.push(RenderElement::new(
                gpu_instance_id,
                material_id,
                mesh_manager.get_mesh_meta_data(mesh.mesh_id),
            ));
        }

        self.entity_to_gpu_instance_block.insert(
            entity,
            GpuInstanceBlock {
                // entity,
                gpu_instance_ids,
                gpu_instance_keys,
            },
        );
    }

    fn update_gpu_instance_block(
        &self,
        entity: Entity,
        visual: &VisualComponent,
        picking_context: &mut PickingIdContext<'_>,
        updater: &mut UniformGPUDataUpdater,
    ) {
        // Color
        let color: (f32, f32, f32, f32) = (
            f32::from(visual.color.r) / 255.0f32,
            f32::from(visual.color.g) / 255.0f32,
            f32::from(visual.color.b) / 255.0f32,
            f32::from(visual.color.a) / 255.0f32,
        );
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());
        instance_color.set_color_blend(visual.color_blend.into());
        self.color_manager
            .update_gpu_data(&entity, &instance_color, updater);

        // Picking
        let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
        picking_data.set_picking_id(picking_context.acquire_picking_id(entity).into());
        self.picking_data_manager
            .update_gpu_data(&entity, &picking_data, updater);
    }

    fn remove_gpu_instance_block(
        &mut self,
        entity: Entity,
        // event_writer: &mut EventWriter<'_, '_, GpuInstanceEvent>,
    ) {
        let gpu_instance_block = self.entity_to_gpu_instance_block.remove(&entity);
        if let Some(mut gpu_instance_block) = gpu_instance_block {
            self.transform_manager.remove_gpu_data(&entity);
            self.color_manager.remove_gpu_data(&entity);
            self.picking_data_manager.remove_gpu_data(&entity);
            for gpu_instance_key in &gpu_instance_block.gpu_instance_keys {
                self.va_table_manager.remove_gpu_data(gpu_instance_key);
            }
            self.removed_gpu_instance_ids
                .append(&mut gpu_instance_block.gpu_instance_ids);

            // event_writer.send(GpuInstanceEvent::Removed(
            //     ,
            // ));
        }
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
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    // mut event_writer: EventWriter<'_, '_, GpuInstanceEvent>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
    instance_query: Query<
        '_,
        '_,
        (Entity, &GlobalTransform, &VisualComponent),
        Changed<VisualComponent>,
    >,
    removed_visual_components: RemovedComponents<'_, VisualComponent>,
    removed_transform_components: RemovedComponents<'_, GlobalTransform>,
) {
    //
    // Clear transient containers
    //
    instance_manager.clear_transient_containers();

    //
    // Unregister all the blocks not needed anymore
    //

    for entity in removed_visual_components.iter() {
        instance_manager.remove_gpu_instance_block(entity);
    }

    for entity in removed_transform_components.iter() {
        instance_manager.remove_gpu_instance_block(entity);
    }

    //
    // TODO(vdbdd): We are going to reconstruct the gpu instances of the changed visuals. First we destroy, then we recreate.
    //  We should just update the block information if the block is already allocated.
    //

    for (entity, _, _) in instance_query.iter() {
        instance_manager.remove_gpu_instance_block(entity);
    }

    //
    // Now, recreate the instance block of entities matching the request
    //

    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    {
        // let mut added_instances = Vec::new();

        for (entity, _, visual) in instance_query.iter() {
            instance_manager.add_gpu_instance_block(
                entity,
                visual,
                &renderer,
                &model_manager,
                &mesh_manager,
                &material_manager,
                &mut missing_visuals_tracker,
                &mut updater,
            );
            // added_instances.push((
            //     material_id,
            //     RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
            // ));
        }

        // event_writer.send(GpuInstanceEvent::Added(added_instances));
    }

    //
    // TODO(vdbdd): this update could be done in a separate system once we don't reconstruct everything on each change.
    //
    {
        let mut picking_context = PickingIdContext::new(&picking_manager);
        for (entity, _, visual) in instance_query.iter() {
            instance_manager.update_gpu_instance_block(
                entity,
                visual,
                &mut picking_context,
                &mut updater,
            );
        }
    }
    renderer.add_update_job_block(updater.job_blocks());

    //
    // Transform
    //
    // let mut world = cgen::cgen_type::Transform::default();
    // world.set_translation(transform.translation.into());
    // world.set_rotation(Vec4::from(transform.rotation).into());
    // world.set_scale(transform.scale.into());

    // transform_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
    // transform_manager.update_gpu_data(&entity, &world, &mut updater);

    //
    // Color
    //
    // let color: (f32, f32, f32, f32) = (
    //     f32::from(visual.color.r) / 255.0f32,
    //     f32::from(visual.color.g) / 255.0f32,
    //     f32::from(visual.color.b) / 255.0f32,
    //     f32::from(visual.color.a) / 255.0f32,
    // );
    // let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
    // instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());
    // instance_color.set_color_blend(visual.color_blend.into());

    // color_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
    // color_manager.update_gpu_data(&entity, &instance_color, &mut updater);

    //
    // Picking part
    //
    // let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
    // picking_data.set_picking_id(picking_context.acquire_picking_id(entity).into());

    // picking_data_manager.alloc_gpu_data(&entity, renderer.static_buffer_allocator());
    // picking_data_manager.update_gpu_data(&entity, &picking_data, &mut updater);

    //
    // Model (might no be ready. it returns a default model)
    // TODO(vdbdd): should be managed at call site (default model depending on some criterias)
    //
    // let (model_meta_data, ready) =
    //     model_manager.get_model_meta_data(visual.model_resource_id.as_ref());
    // if !ready {
    //     warn!(
    //         "Dependency issue. Model {} not loaded for entity {:?}",
    //         visual.model_resource_id.unwrap(),
    //         entity
    //     );
    //     if let Some(model_resource_id) = &visual.model_resource_id {
    //         missing_visuals_tracker.add_resource_entity_dependency(*model_resource_id, entity);
    //     }
    // }

    //
    // Gpu instances
    //

    // let mut added_instances = Vec::with_capacity(model_meta_data.meshes.len());
    // let mut gpu_instance_ids = Vec::new();
    // let mut gpu_instance_keys = Vec::new();
    // let default_material_id = material_manager.get_default_material_id();

    // for mesh in &model_meta_data.meshes {
    //
    // Mesh
    //
    // let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);

    //
    // Material (might not be valid)
    //
    // let material_id = if material_manager.is_material_ready(mesh.material_id) {
    //     mesh.material_id
    // } else {
    //     let material_resource_id = material_manager
    //         .get_material(mesh.material_id)
    //         .resource_id();
    //     warn!(
    //         "Dependency issue. Material {} not ready for entity {:?}",
    //         material_resource_id, entity
    //     );
    //     missing_visuals_tracker
    //         .add_resource_entity_dependency(*material_resource_id, entity);
    //     default_material_id
    // };

    //
    // Gpu instance
    //

    //     let instance_vas = GpuInstanceVas {
    //         submesh_va: mesh_meta_data.mesh_description_offset,
    //         material_va: material_manager.get_material(material_id).va() as u32,
    //         color_va: color_manager.va_for_key(&entity) as u32,
    //         transform_va: transform_manager.va_for_key(&entity) as u32,
    //         picking_data_va: picking_data_manager.va_for_key(&entity) as u32,
    //     };

    //     let (gpu_instance_key, gpu_instance_id) = instance_manager.add_gpu_instance(
    //         entity,
    //         mesh.mesh_id,
    //         renderer.static_buffer_allocator(),
    //         &mut updater,
    //         &instance_vas,
    //     );

    //     gpu_instance_keys.push(gpu_instance_key);
    //     gpu_instance_ids.push(gpu_instance_id);

    //     added_instances.push((
    //         material_id,
    //         RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
    //     ));
    // }

    // event_writer.send(GpuInstanceEvent::Added(added_instances));

    //     instance_manager.add_gpu_instance_block(
    //         entity,
    //         GpuInstanceBlock {
    //             entity,
    //             gpu_instance_ids,
    //             gpu_instance_keys,
    //         },
    //     );
    // }

    // renderer.add_update_job_block(updater.job_blocks());
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn upload_transform_data(
    renderer: Res<'_, Renderer>,
    instance_manager: Res<'_, GpuInstanceManager>,
    query: Query<'_, '_, (Entity, &GlobalTransform, &VisualComponent), Changed<GlobalTransform>>,
) {
    //
    // TODO(vdbdd): to use a parallel for, we need a new API in bevy.
    //

    let transform_count = query.iter().count();
    let block_size = transform_count * std::mem::size_of::<cgen::cgen_type::Transform>();
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), block_size as u64);

    for (entity, transform, _) in query.iter() {
        let mut world = cgen::cgen_type::Transform::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        instance_manager
            .transform_manager
            .update_gpu_data(&entity, &world, &mut updater);
    }

    renderer.add_update_job_block(updater.job_blocks());
}
