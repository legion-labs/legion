use std::collections::BTreeMap;

use lgn_app::App;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{
    Added, Changed, Commands, Component, Entity, Query, RemovedComponents, Res, ResMut,
};
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, CommandBuffer, DeviceContext, Extents3D, Format,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, QueueType, ResourceFlags, ResourceState,
    ResourceUsage, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::{runtime_texture::TextureReferenceType, TextureFormat};
use lgn_tracing::span_fn;

use crate::{
    components::{TextureComponent, TextureData},
    labels::RenderStage,
    Renderer,
};

use super::{IndexAllocator, PersistentDescriptorSetManager};

const BLOCK_SIZE: u32 = 256;

pub enum TextureEvent {}

#[derive(Default, Clone)]
struct UploadTextureJob {
    gpu_texture_id: GpuTextureId,
    texture_data: Option<TextureData>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct GpuTextureId {
    generation: u32,
    index: u32,
}

const INVALID_GENERATION: u32 = 0;
const INVALID_INDEX: u32 = 0;

impl Default for GpuTextureId {
    fn default() -> Self {
        Self {
            generation: INVALID_GENERATION,
            index: INVALID_INDEX,
        }
    }
}

#[derive(Component)]
struct GPUTextureComponent {
    _gpu_texture_id: GpuTextureId,
}

// impl Plugin for TextureManagerPlugin {
//     fn build(&self, app: &mut lgn_app::App) {
//         let texture_manager = TextureManager::new(&self.device_context, 256);
//         let texture_resource_manager = TextureResourceManager::new();
//         app.add_event::<TextureEvent>();
//         app.insert_resource(texture_manager);
//         app.insert_resource(texture_resource_manager);
//         app.add_system_to_stage(RenderStage::Prepare, update_texture_manager);
//         app.add_system_to_stage(RenderStage::Prepare, on_texture_added);
//         app.add_system_to_stage(RenderStage::Prepare, on_texture_modified);
//         app.add_system_to_stage(RenderStage::Prepare, on_texture_removed);
//     }
// }

#[derive(Clone, Copy, PartialEq)]
enum TextureState {
    Invalid,
    QueuedForUpload,
    Ready,
}

#[derive(Clone)]
struct TextureInfo {
    generation: u32,
    state: TextureState,
    bindless_index: Option<u32>,
    texture_view: Option<TextureView>,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            generation: INVALID_GENERATION + 1,
            state: TextureState::Invalid,
            bindless_index: None,
            texture_view: None,
        }
    }
}

pub struct TextureManager {
    device_context: DeviceContext,
    texture_info: Vec<TextureInfo>,
    upload_jobs: Vec<UploadTextureJob>,
    gpu_texture_id_allocator: IndexAllocator,
    bindless_index_allocator: IndexAllocator,
}

impl TextureManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            texture_info: Vec::new(),
            upload_jobs: Vec::new(),
            gpu_texture_id_allocator: IndexAllocator::new(BLOCK_SIZE),
            bindless_index_allocator: IndexAllocator::new(BLOCK_SIZE),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_event::<TextureEvent>();
        app.add_system_to_stage(RenderStage::Prepare, update_texture_manager);
    }

    pub fn is_valid(&self, gpu_texture_id: GpuTextureId) -> bool {
        let index = gpu_texture_id.index as usize;
        if index >= self.texture_info.len() {
            return false;
        }
        let texture_info = &self.texture_info[index];
        let generation = gpu_texture_id.generation;
        texture_info.generation == generation
    }

    pub fn allocate_texture(
        &mut self,
        texture_def: &TextureDef,
        texture_data: &TextureData,
    ) -> GpuTextureId {
        let texture_def = Self::build_texture_def(texture_def);
        let texture_view = self.create_texture_view(&texture_def);
        let index = self.allocate_texture_info();
        let mut texture_info = &mut self.texture_info[index as usize];
        texture_info.texture_view = Some(texture_view);

        let gpu_texture_id = GpuTextureId {
            generation: texture_info.generation,
            index,
        };

        self.queue_for_upload(gpu_texture_id, texture_data);

        gpu_texture_id
    }

    pub fn update_texture(
        &mut self,
        gpu_texture_id: GpuTextureId,
        texture_def: &TextureDef,
        texture_data: &TextureData,
    ) {
        assert!(self.is_valid(gpu_texture_id));

        let texture_def = Self::build_texture_def(texture_def);
        let recreate_texture_view = {
            let current_texture_handle = self.texture_handle(gpu_texture_id);
            let current_texture_def = current_texture_handle.definition();
            current_texture_def != &texture_def
        };
        if recreate_texture_view {
            let texture_view = self.create_texture_view(&texture_def);
            let texture_info = self.texture_info_mut(gpu_texture_id);

            // The previous texture/texture_view is being pushed is the deferred delete queue
            // Should be updated in the persistent descriptor set
            texture_info.texture_view = Some(texture_view);
        }
        self.queue_for_upload(gpu_texture_id, texture_data);
    }

    pub fn release_texture(&mut self, gpu_texture_id: GpuTextureId) {
        assert!(self.is_valid(gpu_texture_id));

        // No need to remove from the upload queue because the gpu_texture_id stored in the upload_job
        // becomes invalid (generation mismatch).

        let texture_info = self.texture_info_mut(gpu_texture_id);
        texture_info.generation += 1;
        texture_info.state = TextureState::Invalid;
        texture_info.texture_view = None;
    }

    pub fn get_bindless_index(&self, gpu_texture_id: GpuTextureId) -> Option<u32> {
        if self.is_valid(gpu_texture_id) {
            let texture_info = self.texture_info(gpu_texture_id);
            texture_info.bindless_index
        } else {
            None
        }
    }

    #[span_fn]
    pub fn update(
        &mut self,
        renderer: &Renderer,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) {
        if self.upload_jobs.is_empty() {
            return;
        }

        let mut upload_jobs = std::mem::take(&mut self.upload_jobs);

        self.upload_textures(renderer, &upload_jobs);
        self.update_persistent_descriptor_set(persistent_descriptor_set_manager, &upload_jobs);

        for upload_job in &upload_jobs {
            let texture_info = self.texture_info_mut(upload_job.gpu_texture_id);
            texture_info.state = TextureState::Ready;
        }

        upload_jobs.resize(0, UploadTextureJob::default());

        self.upload_jobs = upload_jobs;
    }

    #[span_fn]
    fn upload_textures(&mut self, renderer: &Renderer, upload_jobs: &[UploadTextureJob]) {
        let device_context = renderer.device_context();
        let cmd_buffer_pool = renderer.acquire_command_buffer_pool(QueueType::Graphics);
        let cmd_buffer = cmd_buffer_pool.acquire();

        cmd_buffer.begin().unwrap();

        for upload_job in upload_jobs {
            let gpu_texture_id = upload_job.gpu_texture_id;
            if self.is_valid(gpu_texture_id) {
                let texture = self.texture_handle(gpu_texture_id);
                let texture_data = upload_job.texture_data.as_ref().unwrap();
                let mip_slices = texture_data.data();
                for (mip_level, mip_data) in mip_slices.iter().enumerate() {
                    upload_texture_data(
                        device_context,
                        &cmd_buffer,
                        texture,
                        mip_data,
                        mip_level as u8,
                    );
                }
            }
        }

        cmd_buffer.end().unwrap();

        let graphics_queue = renderer.graphics_queue_guard(QueueType::Graphics);

        graphics_queue
            .submit(&[&cmd_buffer], &[], &[], None)
            .unwrap();

        cmd_buffer_pool.release(cmd_buffer);

        renderer.release_command_buffer_pool(cmd_buffer_pool);
    }

    fn update_persistent_descriptor_set(
        &mut self,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
        upload_jobs: &[UploadTextureJob],
    ) {
        for upload_job in upload_jobs {
            let bindless_index = self.bindless_index_allocator.acquire_index();
            let texture_info = self.texture_info_mut(upload_job.gpu_texture_id);
            assert!(texture_info.bindless_index == None);
            texture_info.bindless_index = Some(bindless_index);
            persistent_descriptor_set_manager
                .set_bindless_texture(bindless_index, texture_info.texture_view.as_ref().unwrap());
        }
    }

    fn allocate_texture_info(&mut self) -> u32 {
        let index = self.gpu_texture_id_allocator.acquire_index();

        if index as usize >= self.texture_info.len() {
            let required_size = next_multiple_of(index as usize, BLOCK_SIZE as usize);
            self.texture_info
                .resize(required_size, TextureInfo::default());
        }

        assert!(self.texture_info[index as usize].state == TextureState::Invalid);

        index
    }

    fn build_texture_def(texture_def: &TextureDef) -> TextureDef {
        assert!(texture_def
            .usage_flags
            .contains(ResourceUsage::AS_SHADER_RESOURCE));
        assert_eq!(texture_def.mem_usage, MemoryUsage::GpuOnly);

        let mut result = *texture_def;
        result.usage_flags |= ResourceUsage::AS_TRANSFERABLE;
        result.tiling = TextureTiling::Optimal;

        result
    }

    fn create_texture_view(&self, texture_def: &TextureDef) -> TextureView {
        let texture = self.device_context.create_texture(texture_def);
        texture.create_view(&TextureViewDef::as_shader_resource_view(texture_def))
    }

    fn queue_for_upload(&mut self, gpu_texture_id: GpuTextureId, texture_data: &TextureData) {
        assert!(self.is_valid(gpu_texture_id));

        let current_state = self.texture_state(gpu_texture_id);

        match current_state {
            TextureState::QueuedForUpload => {
                // patch the current upload queue as we know it is safe (mut access)
                for upload_job in &mut self.upload_jobs {
                    if upload_job.gpu_texture_id == gpu_texture_id {
                        upload_job.texture_data = Some(texture_data.clone());
                        break;
                    }
                }
            }
            TextureState::Invalid | TextureState::Ready => {
                self.upload_jobs.push(UploadTextureJob {
                    gpu_texture_id,
                    texture_data: Some(texture_data.clone()),
                });
            }
        }

        let texture_info = self.texture_info_mut(gpu_texture_id);
        texture_info.state = TextureState::QueuedForUpload;
    }

    fn texture_info(&self, gpu_texture_id: GpuTextureId) -> &TextureInfo {
        assert!(self.is_valid(gpu_texture_id));
        &self.texture_info[gpu_texture_id.index as usize]
    }

    fn texture_info_mut(&mut self, gpu_texture_id: GpuTextureId) -> &mut TextureInfo {
        assert!(self.is_valid(gpu_texture_id));
        &mut self.texture_info[gpu_texture_id.index as usize]
    }

    fn texture_handle(&self, gpu_texture_id: GpuTextureId) -> &Texture {
        assert!(self.is_valid(gpu_texture_id));
        let texture_info = self.texture_info(gpu_texture_id);
        texture_info.texture_view.as_ref().unwrap().texture()
    }

    fn texture_state(&self, gpu_texture_id: GpuTextureId) -> TextureState {
        assert!(self.is_valid(gpu_texture_id));
        let texture_info = self.texture_info(gpu_texture_id);
        texture_info.state
    }
}

pub struct TextureResourceManager {
    entity_to_resource_id: BTreeMap<Entity, ResourceTypeAndId>,
    resource_id_to_gpu_texture_id: BTreeMap<ResourceTypeAndId, GpuTextureId>,
}

impl TextureResourceManager {
    pub fn new() -> Self {
        Self {
            entity_to_resource_id: BTreeMap::new(),
            resource_id_to_gpu_texture_id: BTreeMap::new(),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(RenderStage::Prepare, on_texture_added);
        app.add_system_to_stage(RenderStage::Prepare, on_texture_modified);
        app.add_system_to_stage(RenderStage::Prepare, on_texture_removed);
    }

    pub fn allocate_texture(
        &mut self,
        texture_manager: &mut TextureManager,
        entity: Entity,
        texture_component: &TextureComponent,
    ) -> GpuTextureId {
        let texture_def = Self::texture_def_from_texture_component(texture_component);

        let gpu_texture_id =
            texture_manager.allocate_texture(&texture_def, &texture_component.texture_data);

        self.entity_to_resource_id
            .insert(entity, texture_component.texture_id);

        self.resource_id_to_gpu_texture_id
            .insert(texture_component.texture_id, gpu_texture_id);

        gpu_texture_id
    }

    pub fn update_by_entity(
        &mut self,
        texture_manager: &mut TextureManager,
        entity: Entity,
        texture_component: &TextureComponent,
    ) {
        let resource_id = self.entity_to_resource_id.get(&entity).unwrap();

        let gpu_texture_id = self.resource_id_to_gpu_texture_id.get(resource_id).unwrap();

        let texture_def = Self::texture_def_from_texture_component(texture_component);

        texture_manager.update_texture(
            *gpu_texture_id,
            &texture_def,
            &texture_component.texture_data,
        );
    }

    pub fn remove_by_entity(&mut self, texture_manager: &mut TextureManager, entity: Entity) {
        let resource_id = self.entity_to_resource_id.get(&entity).unwrap();

        let gpu_texture_id = self.resource_id_to_gpu_texture_id.get(resource_id).unwrap();

        texture_manager.release_texture(*gpu_texture_id);
    }

    pub fn bindless_index_for_resource_id(
        &self,
        texture_manager: &TextureManager,
        texture_id: &TextureReferenceType,
    ) -> Option<u32> {
        let gpu_texture_id = self.resource_id_to_gpu_texture_id.get(&texture_id.id());
        match gpu_texture_id {
            Some(gpu_texture_id) => texture_manager.get_bindless_index(*gpu_texture_id),
            None => None,
        }
    }

    fn texture_def_from_texture_component(texture_component: &TextureComponent) -> TextureDef {
        let format = match texture_component.format {
            TextureFormat::BC1 => Format::BC1_RGBA_UNORM_BLOCK,
            TextureFormat::BC3 => Format::BC3_UNORM_BLOCK,
            TextureFormat::BC4 => Format::BC4_UNORM_BLOCK,
            TextureFormat::BC7 => Format::BC7_UNORM_BLOCK,
        };

        TextureDef {
            extents: Extents3D {
                width: texture_component.width,
                height: texture_component.height,
                depth: 1,
            },
            array_length: 1,
            mip_count: texture_component.texture_data.mip_count() as u32,
            format,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

fn next_multiple_of(value: usize, multiple: usize) -> usize {
    // todo: replace with value.next_multiple_of asap
    ((value + multiple - 1) / multiple) * multiple
}

fn upload_texture_data(
    device_context: &DeviceContext,
    cmd_buffer: &CommandBuffer,
    texture: &Texture,
    data: &[u8],
    mip_level: u8,
) {
    // todo: this code must be completly rewritten (-> upload manager)
    let staging_buffer = device_context.create_buffer(&BufferDef::for_staging_buffer_data(
        data,
        ResourceUsage::empty(),
    ));

    let alloc_def = MemoryAllocationDef {
        memory_usage: MemoryUsage::CpuToGpu,
        always_mapped: true,
    };

    let buffer_memory = MemoryAllocation::from_buffer(device_context, &staging_buffer, &alloc_def);

    buffer_memory.copy_to_host_visible_buffer(data);

    // todo: not needed
    cmd_buffer.cmd_resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::UNDEFINED,
            ResourceState::COPY_DST,
        )],
    );

    cmd_buffer.cmd_copy_buffer_to_texture(
        &staging_buffer,
        texture,
        &CmdCopyBufferToTextureParams {
            mip_level,
            ..CmdCopyBufferToTextureParams::default()
        },
    );

    // todo: not needed
    cmd_buffer.cmd_resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::COPY_DST,
            ResourceState::SHADER_RESOURCE,
        )],
    );
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_added(
    mut commands: Commands<'_, '_>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut texture_resource_manager: ResMut<'_, TextureResourceManager>,
    q_added_textures: Query<'_, '_, (Entity, &TextureComponent), Added<TextureComponent>>,
) {
    for (entity, texture_component) in q_added_textures.iter() {
        let gpu_texture_id = texture_resource_manager.allocate_texture(
            &mut texture_manager,
            entity,
            texture_component,
        );

        commands.entity(entity).insert(GPUTextureComponent {
            _gpu_texture_id: gpu_texture_id,
        });
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_modified(
    mut texture_manager: ResMut<'_, TextureManager>,
    mut texture_resource_manager: ResMut<'_, TextureResourceManager>,
    q_modified_textures: Query<
        '_,
        '_,
        (Entity, &TextureComponent, &GPUTextureComponent),
        Changed<TextureComponent>,
    >,
) {
    for (entity, texture_component, _) in q_modified_textures.iter() {
        texture_resource_manager.update_by_entity(&mut texture_manager, entity, texture_component);
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_removed(
    removed_entities: RemovedComponents<'_, TextureComponent>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut texture_resource_manager: ResMut<'_, TextureResourceManager>,
) {
    for removed_entity in removed_entities.iter() {
        texture_resource_manager.remove_by_entity(&mut texture_manager, removed_entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn update_texture_manager(
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    texture_manager.update(&renderer, &mut persistent_descriptor_set_manager);
}
