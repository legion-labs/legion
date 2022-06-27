use std::{collections::BTreeMap, sync::Arc};

use lgn_app::App;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::{
    prelude::{Changed, Entity, Query, RemovedComponents, Res},
    schedule::{SystemLabel, SystemSet},
};
use lgn_graphics_api::{BufferView, BufferViewDef, ResourceUsage, VertexBufferBinding};

use lgn_math::Vec4;

use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen,
    components::VisualComponent,
    core::{BinaryWriter, GpuUploadManager, RenderCommandBuilder},
    labels::RenderStage,
    picking::{PickingIdContext, PickingManager},
    resources::{
        GpuDataAllocation, GpuDataManager, MeshManager, RenderModel, StaticBufferAllocation,
        StaticBufferView, UnifiedStaticBuffer, UpdateUnifiedStaticBufferCommand,
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
type GpuEntityTransformManager = GpuDataManager<Entity, cgen::cgen_type::TransformData>;
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
    gpu_instance_ids: Vec<GpuInstanceId>,
    gpu_instance_keys: Vec<GpuInstanceKey>,
}

struct GpuVaTableForGpuInstance {
    static_allocation: StaticBufferAllocation,
    static_buffer_view: StaticBufferView,
}

impl GpuVaTableForGpuInstance {
    pub fn new(gpu_heap: &UnifiedStaticBuffer) -> Self {
        let element_count = 1024 * 1024;
        let element_size = std::mem::size_of::<u32>() as u64;
        let static_allocation = gpu_heap.allocate(
            element_count * element_size,
            ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_VERTEX_BUFFER,
        );

        let buffer_view = static_allocation.create_view(BufferViewDef::as_structured_buffer(
            element_count,
            element_size,
            true,
        ));

        Self {
            static_allocation,
            static_buffer_view: buffer_view,
        }
    }

    pub fn set_va_table_address_for_gpu_instance(
        &self,
        render_commands: &mut RenderCommandBuilder,
        gpu_data_allocation: GpuDataAllocation,
    ) {
        let offset_for_gpu_instance =
            self.static_allocation.byte_offset() + u64::from(gpu_data_allocation.index()) * 4;

        let va = u32::try_from(gpu_data_allocation.gpuheap_addr()).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(&va);

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: binary_writer.take(),
            dst_offset: offset_for_gpu_instance,
        });
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        self.static_allocation.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self) -> &BufferView {
        self.static_buffer_view.buffer_view()
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
    pub fn new(gpu_heap: &UnifiedStaticBuffer, gpu_upload_manager: &GpuUploadManager) -> Self {
        Self {
            // TODO(vdbdd): as soon as we have a stable ID, we can move the transforms in their own manager.
            transform_manager: GpuEntityTransformManager::new(gpu_heap, 1024, gpu_upload_manager),
            color_manager: GpuEntityColorManager::new(gpu_heap, 256, gpu_upload_manager),
            picking_data_manager: GpuPickingDataManager::new(gpu_heap, 1024, gpu_upload_manager),
            va_table_manager: GpuVaTableManager::new(gpu_heap, 4096, gpu_upload_manager),
            va_table_adresses: GpuVaTableForGpuInstance::new(gpu_heap),
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

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        self.va_table_adresses.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self) -> &BufferView {
        self.va_table_adresses.structured_buffer_view()
    }

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
        mesh_manager: &MeshManager,
        render_commands: &mut RenderCommandBuilder,
        picking_context: &mut PickingIdContext<'_>,
        asset_registry: &AssetRegistry,
    ) {
        assert!(!self.entity_to_gpu_instance_block.contains_key(&entity));

        // Transform are updated in their own system
        {
            self.transform_manager.alloc_gpu_data(&entity);
        }
        // Color are updated in the update function
        {
            self.color_manager.alloc_gpu_data(&entity);
        }
        // Picking is allocated and updated at creation time
        {
            self.picking_data_manager.alloc_gpu_data(&entity);
            let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
            picking_data.set_picking_id(picking_context.acquire_picking_id(entity).into());
            self.picking_data_manager
                .update_gpu_data(&entity, &picking_data, render_commands);
        }

        //
        // Model (might no be ready. it returns a default model)
        // TODO(vdbdd): should be managed at call site (default model depending on some criterias)
        //

        let render_model_handle = asset_registry
            .lookup::<RenderModel>(visual.model_resource_id())
            .expect("Must been loaded");
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
            // Material (might not be valid)
            //

            //
            // Gpu instance
            //

            let instance_vas = GpuInstanceVas {
                submesh_va: render_mesh.mesh_description_offset,
                material_va: mesh.material_va as u32,
                color_va: self.color_manager.gpuheap_addr_for_key(&entity) as u32,
                transform_va: self.transform_manager.gpuheap_addr_for_key(&entity) as u32,
                picking_data_va: self.picking_data_manager.gpuheap_addr_for_key(&entity) as u32,
            };

            let gpu_instance_key = GpuInstanceKey { entity, mesh_index };
            gpu_instance_keys.push(gpu_instance_key);

            let gpu_data_allocation = self.va_table_manager.alloc_gpu_data(&gpu_instance_key);

            let gpu_instance_id = gpu_data_allocation.index().into();
            gpu_instance_ids.push(gpu_instance_id);

            self.va_table_adresses
                .set_va_table_address_for_gpu_instance(render_commands, gpu_data_allocation);

            let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
            gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
            gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());
            gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
            gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
            gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

            let mut binary_writer = BinaryWriter::new();
            binary_writer.write(&gpu_instance_va_table);

            render_commands.push(UpdateUnifiedStaticBufferCommand {
                src_buffer: binary_writer.take(),
                dst_offset: gpu_data_allocation.gpuheap_addr(),
            });

            self.added_render_elements.push(RenderElement::new(
                gpu_instance_id,
                mesh.material_id,
                render_mesh,
            ));
        }

        self.entity_to_gpu_instance_block.insert(
            entity,
            GpuInstanceBlock {
                gpu_instance_ids,
                gpu_instance_keys,
            },
        );
    }

    fn update_gpu_instance_block(
        &self,
        entity: Entity,
        transform: &GlobalTransform,
        visual: &VisualComponent,
        render_commands: &mut RenderCommandBuilder,
    ) {
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color((u32::from(visual.color())).into());
        instance_color.set_color_blend(visual.color_blend().into());
        self.color_manager
            .update_gpu_data(&entity, &instance_color, render_commands);

        let mut world = cgen::cgen_type::TransformData::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        self.transform_manager
            .update_gpu_data(&entity, &world, render_commands);
    }

    fn remove_gpu_instance_block(&mut self, entity: Entity) {
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
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    instance_query: Query<
        '_,
        '_,
        (Entity, &GlobalTransform, &VisualComponent),
        Changed<VisualComponent>,
    >,
    removed_visual_components: RemovedComponents<'_, VisualComponent>,
    removed_transform_components: RemovedComponents<'_, GlobalTransform>,
) {
    let picking_manager = renderer.render_resources().get::<PickingManager>();
    let mesh_manager = renderer.render_resources().get::<MeshManager>();
    let mut instance_manager = renderer.render_resources().get_mut::<GpuInstanceManager>();

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

    let mut render_commands = renderer.render_command_builder();

    {
        let mut picking_context = PickingIdContext::new(&picking_manager);

        for (entity, _, visual) in instance_query.iter() {
            instance_manager.add_gpu_instance_block(
                entity,
                visual,
                &mesh_manager,
                &mut render_commands,
                &mut picking_context,
                asset_registry.as_ref(),
            );
        }
    }

    // TODO(vdbdd): this update could be done in a separate system once we don't reconstruct everything on each change.
    {
        for (entity, transform, visual) in instance_query.iter() {
            instance_manager.update_gpu_instance_block(
                entity,
                transform,
                visual,
                &mut render_commands,
            );
        }
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn upload_transform_data(
    renderer: Res<'_, Renderer>,
    query: Query<'_, '_, (Entity, &GlobalTransform, &VisualComponent), Changed<GlobalTransform>>,
) {
    let instance_manager = renderer.render_resources().get::<GpuInstanceManager>();
    let mut render_commands = renderer.render_command_builder();

    for (entity, transform, _) in query.iter() {
        let mut world = cgen::cgen_type::TransformData::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        instance_manager
            .transform_manager
            .update_gpu_data(&entity, &world, &mut render_commands);
    }
}
