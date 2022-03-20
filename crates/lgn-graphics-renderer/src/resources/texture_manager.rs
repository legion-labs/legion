use std::{collections::BTreeMap, sync::Arc};

use lgn_app::App;
use lgn_data_runtime::{AssetRegistry, AssetRegistryEvent, Resource, ResourceTypeAndId};
use lgn_ecs::{prelude::*, schedule::SystemLabel};
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, CommandBuffer, DeviceContext, Extents3D, Format,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, QueueType, ResourceFlags, ResourceState,
    ResourceUsage, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::runtime_texture::Texture as TextureAsset;
use lgn_graphics_data::TextureFormat;
use lgn_tracing::span_fn;

use crate::{components::TextureData, labels::RenderStage, Renderer};

use super::PersistentDescriptorSetManager;

pub enum TextureEvent {
    StateChanged(Vec<ResourceTypeAndId>),
}

struct UploadTextureJob {
    texture_id: ResourceTypeAndId,
    texture_data: Option<TextureData>,
}

struct RemoveTextureJob {
    texture_id: ResourceTypeAndId,
}

enum TextureJob {
    Upload(UploadTextureJob),
    Remove(RemoveTextureJob),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TextureState {
    Invalid,
    QueuedForUpload,
    Ready,
}

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum TextureManagerLabel {
    UpdateDone,
}

#[derive(Clone)]
struct TextureInfo {
    state: TextureState,
    texture_id: ResourceTypeAndId,
    bindless_index: Option<u32>,
    texture_view: TextureView,
}

pub struct TextureManager {
    device_context: DeviceContext,
    texture_infos: BTreeMap<ResourceTypeAndId, TextureInfo>,
    // todo: use some kind of queue maybe?
    texture_jobs: Vec<TextureJob>,
}

impl TextureManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            texture_infos: BTreeMap::new(),
            texture_jobs: Vec::new(),
            //texture_id_to_texture_component: BTreeMap::new(),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_event::<TextureEvent>();

        app.add_system(on_texture_events);

        app.add_system_to_stage(
            RenderStage::Prepare,
            apply_changes.after(TextureManagerLabel::UpdateDone),
        );
    }

    pub fn allocate_texture(
        &mut self,
        resource_id: ResourceTypeAndId,
        texture_asset: &TextureAsset,
    ) {
        let texture_def = Self::texture_def_from_texture_asset(texture_asset);

        let texture_view = self.create_texture_view(&texture_def);

        self.texture_infos.insert(
            resource_id,
            TextureInfo {
                state: TextureState::Invalid,
                texture_id: resource_id,
                bindless_index: None,
                texture_view,
            },
        );

        let texture_mips = texture_asset // TODO: Avoid cloning in the future
            .texture_data
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();

        self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
            texture_id: resource_id,
            texture_data: Some(TextureData::from_slices(&texture_mips)),
        }));

        let texture_info = self.texture_info_mut(resource_id);
        texture_info.state = TextureState::QueuedForUpload;
    }

    pub fn update_texture(&mut self, resource_id: ResourceTypeAndId, texture_asset: &TextureAsset) {
        // TODO(vdbdd): not tested
        assert_eq!(self.texture_info(resource_id).state, TextureState::Ready);

        let texture_def = Self::texture_def_from_texture_asset(texture_asset);

        let recreate_texture_view = {
            let texture_info = self.texture_info(resource_id);
            let current_texture_handle = texture_info.texture_view.texture();
            let current_texture_def = current_texture_handle.definition();
            *current_texture_def != texture_def
        };

        if recreate_texture_view {
            let texture_view = self.create_texture_view(&texture_def);
            let texture_info = self.texture_info_mut(resource_id);
            texture_info.texture_view = texture_view;
        }

        let texture_mips = texture_asset
            .texture_data
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();

        self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
            texture_id: resource_id,
            texture_data: Some(TextureData::from_slices(&texture_mips)),
        }));

        let texture_info = self.texture_info_mut(resource_id);
        texture_info.state = TextureState::QueuedForUpload;
    }

    pub fn remove_texture(&mut self, resource_id: ResourceTypeAndId) {
        // TODO(vdbdd): not tested
        assert_eq!(self.texture_info(resource_id).state, TextureState::Ready);

        let texture_id = self.texture_info(resource_id).texture_id;
        self.texture_jobs
            .push(TextureJob::Remove(RemoveTextureJob { texture_id }));

        self.texture_infos.remove(&resource_id);
    }

    pub fn bindless_index_for_resource_id(&self, texture_id: &ResourceTypeAndId) -> Option<u32> {
        let texture_info = self.texture_infos.get(texture_id);
        texture_info.map(|ti| ti.bindless_index).and_then(|ti| ti)
    }

    #[span_fn]
    pub fn apply_changes(
        &mut self,
        renderer: &Renderer,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Vec<ResourceTypeAndId> {
        if self.texture_jobs.is_empty() {
            return Vec::new();
        }

        // TODO(vdbdd): remove this heap allocation
        let mut state_changed_list = Vec::with_capacity(self.texture_jobs.len());
        let mut texture_jobs = std::mem::take(&mut self.texture_jobs);

        for texture_job in &texture_jobs {
            match texture_job {
                TextureJob::Upload(upload_job) => {
                    let bindless_index = self.texture_info(upload_job.texture_id).bindless_index;

                    if let Some(bindless_index) = bindless_index {
                        persistent_descriptor_set_manager.unset_bindless_texture(bindless_index);
                    }

                    self.upload_texture(renderer, upload_job);

                    let texture_info = self.texture_info_mut(upload_job.texture_id);
                    texture_info.state = TextureState::Ready;
                    texture_info.bindless_index = Some(
                        persistent_descriptor_set_manager
                            .set_bindless_texture(&texture_info.texture_view),
                    );

                    state_changed_list.push(texture_info.texture_id);
                }
                TextureJob::Remove(remove_job) => {
                    // TODO(vdbdd): not tested
                    let texture_info = self.texture_infos.get_mut(&remove_job.texture_id).unwrap();

                    let bindless_index = texture_info.bindless_index.unwrap();
                    persistent_descriptor_set_manager.unset_bindless_texture(bindless_index);

                    state_changed_list.push(remove_job.texture_id);
                }
            }
        }

        texture_jobs.clear();

        self.texture_jobs = texture_jobs;

        state_changed_list
    }

    fn is_valid(&self, resource_id: ResourceTypeAndId) -> bool {
        self.texture_infos.contains_key(&resource_id)
    }

    #[span_fn]
    fn upload_texture(&mut self, renderer: &Renderer, upload_job: &UploadTextureJob) {
        let device_context = renderer.device_context();
        let cmd_buffer_pool = renderer.acquire_command_buffer_pool(QueueType::Graphics);
        let cmd_buffer = cmd_buffer_pool.acquire();

        cmd_buffer.begin().unwrap();

        // let gpu_texture_id = upload_job.gpu_texture_id;
        let resource_id = upload_job.texture_id;
        let texture_info = self.texture_infos.get(&resource_id).unwrap();
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

    fn texture_info(&self, resource_id: ResourceTypeAndId) -> &TextureInfo {
        assert!(self.is_valid(resource_id));
        self.texture_infos.get(&resource_id).unwrap()
    }

    fn texture_info_mut(&mut self, resource_id: ResourceTypeAndId) -> &mut TextureInfo {
        assert!(self.is_valid(resource_id));
        self.texture_infos.get_mut(&resource_id).unwrap()
    }

    fn texture_def_from_texture_asset(texture_asset: &TextureAsset) -> TextureDef {
        let format = match texture_asset.format {
            TextureFormat::BC1 => {
                if texture_asset.srgb {
                    Format::BC1_RGBA_SRGB_BLOCK
                } else {
                    Format::BC1_RGBA_UNORM_BLOCK
                }
            }
            TextureFormat::BC3 => {
                if texture_asset.srgb {
                    Format::BC3_SRGB_BLOCK
                } else {
                    Format::BC3_UNORM_BLOCK
                }
            }
            TextureFormat::BC4 => {
                assert!(!texture_asset.srgb);
                Format::BC4_UNORM_BLOCK
            }
            TextureFormat::BC7 => {
                if texture_asset.srgb {
                    Format::BC7_SRGB_BLOCK
                } else {
                    Format::BC7_UNORM_BLOCK
                }
            }
        };

        TextureDef {
            extents: Extents3D {
                width: texture_asset.width,
                height: texture_asset.height,
                depth: 1,
            },
            array_length: 1,
            mip_count: texture_asset.texture_data.len() as u32,
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
        //
        // TODO(vdbdd): this code shoud be moved (-> upload manager)
        // Motivations:
        // - Here the buffer is constantly reallocated
        // - Almost same code for buffer and texture
        // - Leverage the Copy queue
        //
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
fn on_texture_events(
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut asset_loaded_events: EventReader<'_, '_, AssetRegistryEvent>,
) {
    for asset_loaded_event in asset_loaded_events.iter() {
        match asset_loaded_event {
            AssetRegistryEvent::AssetLoaded(resource_id)
                if resource_id.kind == lgn_graphics_data::runtime_texture::Texture::TYPE =>
            {
                if let Some(texture_asset) =
                    asset_registry.get_untyped(*resource_id).and_then(|handle| {
                        handle.get::<lgn_graphics_data::runtime_texture::Texture>(&asset_registry)
                    })
                {
                    texture_manager.allocate_texture(*resource_id, &texture_asset);
                }
            }
            AssetRegistryEvent::AssetChanged(resource_id)
                if resource_id.kind == lgn_graphics_data::runtime_texture::Texture::TYPE =>
            {
                if let Some(texture_asset) =
                    asset_registry.get_untyped(*resource_id).and_then(|handle| {
                        handle.get::<lgn_graphics_data::runtime_texture::Texture>(&asset_registry)
                    })
                {
                    texture_manager.update_texture(*resource_id, &texture_asset);
                }
            }
            AssetRegistryEvent::AssetUnloaded(resource_id)
                if resource_id.kind == lgn_graphics_data::runtime_texture::Texture::TYPE =>
            {
                texture_manager.remove_texture(*resource_id);
            }
            _ => (),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
#[span_fn]
fn apply_changes(
    mut event_writer: EventWriter<'_, '_, TextureEvent>,
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    // todo: must be send some events to refresh the material
    let state_changed_list =
        texture_manager.apply_changes(&renderer, &mut persistent_descriptor_set_manager);
    if !state_changed_list.is_empty() {
        event_writer.send(TextureEvent::StateChanged(state_changed_list));
    }
}
