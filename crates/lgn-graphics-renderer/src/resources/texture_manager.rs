use std::collections::BTreeMap;

use lgn_app::App;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::{prelude::*, schedule::SystemLabel};
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, CommandBuffer, DeviceContext, Extents3D, Format,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, QueueType, ResourceFlags, ResourceState,
    ResourceUsage, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::TextureFormat;
use lgn_tracing::span_fn;

use crate::{
    components::{TextureComponent, TextureData},
    labels::RenderStage,
    Renderer,
};

use super::PersistentDescriptorSetManager;

pub enum TextureEvent {
    StateChanged(Vec<ResourceTypeAndId>),
}

struct UploadTextureJob {
    texture_id: ResourceTypeAndId,
    texture_data: Option<TextureData>,
}

enum TextureJob {
    Upload(UploadTextureJob),
}

#[derive(Component)]
struct GPUTextureComponent;

#[derive(Clone, Copy, PartialEq)]
enum TextureState {
    Invalid,
    QueuedForUpload,
    Ready,
}

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum TextureManagerLabel {
    Done,
}

#[derive(Clone)]
struct TextureInfo {
    state: TextureState,
    bindless_index: Option<u32>,
    texture_view: TextureView,
}

pub struct TextureManager {
    device_context: DeviceContext,
    texture_infos: BTreeMap<ResourceTypeAndId, TextureInfo>,
    // todo: use some kind of queue maybe?
    texture_jobs: Vec<TextureJob>,
    entity_to_texture_id: BTreeMap<Entity, ResourceTypeAndId>,
}

impl TextureManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            texture_infos: BTreeMap::new(),
            texture_jobs: Vec::new(),
            entity_to_texture_id: BTreeMap::new(),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_event::<TextureEvent>();

        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(on_texture_added)
                .with_system(on_texture_modified)
                .with_system(on_texture_removed)
                .label(TextureManagerLabel::Done),
        );

        app.add_system_to_stage(
            RenderStage::Prepare,
            update_texture_manager.after(TextureManagerLabel::Done),
        );
    }

    pub fn is_valid(&self, texture_id: &ResourceTypeAndId) -> bool {
        self.texture_infos.contains_key(texture_id)
    }

    pub fn allocate_texture(&mut self, entity: Entity, texture_component: &TextureComponent) {
        let texture_def = Self::texture_def_from_texture_component(texture_component);

        let texture_view = self.create_texture_view(&texture_def);

        self.texture_infos.insert(
            texture_component.texture_id,
            TextureInfo {
                state: TextureState::Invalid,
                bindless_index: None,
                texture_view,
            },
        );

        self.queue_for_upload(
            &texture_component.texture_id,
            &texture_component.texture_data,
        );

        self.entity_to_texture_id
            .insert(entity, texture_component.texture_id);
    }

    pub fn update_texture(
        &mut self,
        texture_id: &ResourceTypeAndId,
        texture_def: &TextureDef,
        texture_data: &TextureData,
    ) {
        // todo: untested

        assert!(self.is_valid(texture_id));

        let recreate_texture_view = {
            let current_texture_handle = self.texture_handle(texture_id);
            let current_texture_def = current_texture_handle.definition();
            current_texture_def != texture_def
        };
        if recreate_texture_view {
            let texture_view = self.create_texture_view(texture_def);
            let texture_info = self.texture_info_mut(texture_id);
            // The previous texture/texture_view is being pushed is the deferred delete queue
            // Should be updated in the persistent descriptor set
            texture_info.texture_view = texture_view;
        }
        self.queue_for_upload(texture_id, texture_data);
    }

    pub fn update_by_entity(&mut self, entity: Entity, texture_component: &TextureComponent) {
        let texture_id = *self.entity_to_texture_id.get(&entity).unwrap();

        let texture_def = Self::texture_def_from_texture_component(texture_component);

        self.update_texture(&texture_id, &texture_def, &texture_component.texture_data);
    }

    pub fn remove_by_entity(&mut self, entity: Entity) {
        let texture_id = *self.entity_to_texture_id.get(&entity).unwrap();

        self.texture_infos.remove(&texture_id);
    }

    pub fn bindless_index_for_resource_id(&self, texture_id: &ResourceTypeAndId) -> Option<u32> {
        let texture_info = self.texture_infos.get(texture_id);
        texture_info.map(|ti| ti.bindless_index).and_then(|ti| ti)
    }

    #[span_fn]
    pub fn update(
        &mut self,
        renderer: &Renderer,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Vec<ResourceTypeAndId> {
        if self.texture_jobs.is_empty() {
            return Vec::new();
        }

        let mut state_changed_list = Vec::with_capacity(self.texture_jobs.len());
        let mut texture_jobs = std::mem::take(&mut self.texture_jobs);

        for texture_job in &texture_jobs {
            match texture_job {
                TextureJob::Upload(upload_job) => {
                    self.upload_texture(renderer, upload_job);

                    let texture_info = self.texture_infos.get_mut(&upload_job.texture_id).unwrap();
                    texture_info.state = TextureState::Ready;
                    texture_info.bindless_index = Some(
                        persistent_descriptor_set_manager
                            .set_bindless_texture(&texture_info.texture_view),
                    );

                    state_changed_list.push(upload_job.texture_id);
                }
            }
        }

        texture_jobs.clear();

        self.texture_jobs = texture_jobs;

        state_changed_list
    }

    #[span_fn]
    fn upload_texture(&mut self, renderer: &Renderer, upload_job: &UploadTextureJob) {
        let device_context = renderer.device_context();
        let cmd_buffer_pool = renderer.acquire_command_buffer_pool(QueueType::Graphics);
        let cmd_buffer = cmd_buffer_pool.acquire();

        cmd_buffer.begin().unwrap();

        // let gpu_texture_id = upload_job.gpu_texture_id;
        let texture_id = upload_job.texture_id;
        let texture_info = self.texture_infos.get(&texture_id).unwrap();
        {
            let texture = texture_info.texture_view.texture();
            let texture_data = upload_job.texture_data.as_ref().unwrap();
            let mip_slices = texture_data.data();
            for (mip_level, mip_data) in mip_slices.iter().enumerate() {
                Self::upload_texture_data(
                    device_context,
                    &cmd_buffer,
                    texture,
                    mip_data,
                    mip_level as u8,
                );
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

    fn create_texture_view(&self, texture_def: &TextureDef) -> TextureView {
        // todo: bundle all the default views in the resource instead of having them separatly
        let texture = self.device_context.create_texture(texture_def);
        texture.create_view(&TextureViewDef::as_shader_resource_view(texture_def))
    }

    fn queue_for_upload(&mut self, texture_id: &ResourceTypeAndId, texture_data: &TextureData) {
        assert!(self.is_valid(texture_id));

        let current_state = self.texture_state(texture_id);

        match current_state {
            TextureState::QueuedForUpload => {
                // patch the current upload queue as we know it is safe (mut access)
                for texture_job in &mut self.texture_jobs {
                    match texture_job {
                        TextureJob::Upload(upload_job) => {
                            if upload_job.texture_id == *texture_id {
                                upload_job.texture_data = Some(texture_data.clone());
                                break;
                            }
                        }
                    }
                }
            }
            TextureState::Invalid | TextureState::Ready => {
                self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
                    texture_id: *texture_id,
                    texture_data: Some(texture_data.clone()),
                }));
            }
        }

        let texture_info = self.texture_info_mut(texture_id);
        texture_info.state = TextureState::QueuedForUpload;
    }

    fn texture_info(&self, texture_id: &ResourceTypeAndId) -> &TextureInfo {
        assert!(self.is_valid(texture_id));
        self.texture_infos.get(texture_id).unwrap()
    }

    fn texture_info_mut(&mut self, texture_id: &ResourceTypeAndId) -> &mut TextureInfo {
        assert!(self.is_valid(texture_id));
        self.texture_infos.get_mut(texture_id).unwrap()
    }

    fn texture_handle(&self, texture_id: &ResourceTypeAndId) -> &Texture {
        assert!(self.is_valid(texture_id));
        let texture_info = self.texture_info(texture_id);
        texture_info.texture_view.texture()
    }

    fn texture_state(&self, texture_id: &ResourceTypeAndId) -> TextureState {
        assert!(self.is_valid(texture_id));
        let texture_info = self.texture_info(texture_id);
        texture_info.state
    }

    fn texture_def_from_texture_component(texture_component: &TextureComponent) -> TextureDef {
        let format = match texture_component.format {
            TextureFormat::BC1 => {
                if texture_component.srgb {
                    Format::BC1_RGBA_SRGB_BLOCK
                } else {
                    Format::BC1_RGBA_UNORM_BLOCK
                }
            }
            TextureFormat::BC3 => {
                if texture_component.srgb {
                    Format::BC3_SRGB_BLOCK
                } else {
                    Format::BC3_UNORM_BLOCK
                }
            }
            TextureFormat::BC4 => {
                assert!(!texture_component.srgb);
                Format::BC4_UNORM_BLOCK
            }
            TextureFormat::BC7 => {
                if texture_component.srgb {
                    Format::BC7_SRGB_BLOCK
                } else {
                    Format::BC7_UNORM_BLOCK
                }
            }
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

        let buffer_memory =
            MemoryAllocation::from_buffer(device_context, &staging_buffer, &alloc_def);

        buffer_memory.copy_to_host_visible_buffer(data);

        // todo: not needed here
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

        // todo: not needed here
        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                texture,
                ResourceState::COPY_DST,
                ResourceState::SHADER_RESOURCE,
            )],
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_added(
    mut commands: Commands<'_, '_>,
    mut texture_manager: ResMut<'_, TextureManager>,
    q_added_textures: Query<'_, '_, (Entity, &TextureComponent), Added<TextureComponent>>,
) {
    if q_added_textures.is_empty() {
        return;
    }

    for (entity, texture_component) in q_added_textures.iter() {
        texture_manager.allocate_texture(entity, texture_component);

        commands.entity(entity).insert(GPUTextureComponent);
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_modified(
    mut texture_manager: ResMut<'_, TextureManager>,
    q_modified_textures: Query<
        '_,
        '_,
        (Entity, &TextureComponent, &GPUTextureComponent),
        Changed<TextureComponent>,
    >,
) {
    if q_modified_textures.is_empty() {
        return;
    }

    for (entity, texture_component, _) in q_modified_textures.iter() {
        texture_manager.update_by_entity(entity, texture_component);
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn on_texture_removed(
    removed_entities: RemovedComponents<'_, TextureComponent>,
    mut texture_manager: ResMut<'_, TextureManager>,
) {
    // todo: must be send some events to refresh the material
    for removed_entity in removed_entities.iter() {
        texture_manager.remove_by_entity(removed_entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn update_texture_manager(
    mut event_writer: EventWriter<'_, '_, TextureEvent>,
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    // todo: must be send some events to refresh the material
    let state_changed_list =
        texture_manager.update(&renderer, &mut persistent_descriptor_set_manager);
    if !state_changed_list.is_empty() {
        event_writer.send(TextureEvent::StateChanged(state_changed_list));
    }
}
