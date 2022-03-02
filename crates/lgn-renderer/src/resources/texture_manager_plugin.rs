use std::{collections::BTreeMap, rc::Weak, sync::Arc};

use lgn_app::{CoreStage, Plugin};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{
    Added, Changed, Commands, Component, Entity, Query, RemovedComponents, Res, ResMut,
};
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, DeviceContext, Extents3D, Format, MemoryAllocation,
    MemoryAllocationDef, MemoryUsage, ResourceFlags, ResourceState, ResourceUsage, Texture,
    TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::{runtime_texture::TextureReferenceType, TextureFormat};
use lgn_tracing::span_fn;

use crate::{
    components::{TextureComponent, TextureData},
    hl_gfx_api::HLCommandBuffer,
    labels::RenderStage,
    RenderContext, Renderer,
};

use super::{
    DescriptorHeapManager, IndexAllocator, PersistentDescriptorSetManager, PipelineManager,
};
use strum::{EnumCount, IntoEnumIterator};

#[derive(strum::EnumCount, strum::EnumIter)]
pub enum MaterialTextureType {
    Albedo,
    Normal,
    Metalness,
    Roughness,
}

#[derive(Default, Clone)]
struct UploadTextureJob {
    gpu_texture_id: GpuTextureId,
    texture_data: Option<TextureData>,
}

#[derive(Clone, Copy)]
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
    gpu_texture_id: GpuTextureId,
}

pub struct TextureManagerPlugin {
    device_context: DeviceContext,
}

impl TextureManagerPlugin {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
        }
    }
}

impl Plugin for TextureManagerPlugin {
    fn build(&self, app: &mut lgn_app::App) {
        let mut texture_manager = TextureManager::new(&self.device_context, 256);
        let texture_resource_manager = TextureResourceManager::new(&mut texture_manager);
        app.insert_resource(texture_manager);
        app.insert_resource(texture_resource_manager);
        app.add_startup_system(on_startup);
        app.add_system_to_stage(CoreStage::PostUpdate, on_texture_added);
        app.add_system_to_stage(CoreStage::PostUpdate, on_texture_modified);
        app.add_system_to_stage(CoreStage::PostUpdate, on_texture_removed);
        app.add_system_to_stage(RenderStage::Prepare, upload_textures);
        app.add_system_to_stage(RenderStage::Prepare, update_persistent_descriptor_set);
    }
}

#[derive(Clone, PartialEq)]
enum TextureState {
    Invalid,
    QueuedForUpload,
    Uploaded,
    Valid,
}

#[derive(Clone)]
struct TextureInfo {
    generation: u32,
    state: TextureState,
    texture_view: Option<TextureView>,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            generation: INVALID_GENERATION + 1,
            state: TextureState::Invalid,
            texture_view: None,
        }
    }
}

pub struct TextureManager {
    // textures: Vec<Texture>,
    block_size: u32,
    device_context: DeviceContext,
    texture_info: Vec<TextureInfo>,
    // bindless_array: Vec<TextureView>,
    upload_jobs: Vec<UploadTextureJob>,
    index_allocator: IndexAllocator,
    // ref_to_gpu_id: BTreeMap<ResourceTypeAndId, u32>,
    // entity_map: BTreeMap<Entity, u32>,
    // default_texture_id: Option<u32>,
}

impl TextureManager {
    pub fn new(device_context: &DeviceContext, block_size: u32) -> Self {
        // let texture_def = TextureDef {
        //     extents: Extents3D {
        //         width: 4,
        //         height: 4,
        //         depth: 1,
        //     },
        //     array_length: 1,
        //     mip_count: 1,
        //     format: Format::R8G8B8A8_UNORM,
        //     usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
        //     resource_flags: ResourceFlags::empty(),
        //     mem_usage: MemoryUsage::GpuOnly,
        //     tiling: TextureTiling::Linear,
        // };

        let mut index_allocator = IndexAllocator::new(block_size);

        // let default_texture_id = index_allocator.acquire_index();

        // let default_black_texture = device_context.create_texture(&texture_def);

        // let default_black_texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        // let descriptor_array =
        //     vec![default_black_texture.create_view(&default_black_texture_view_def,); array_size];

        // let mut texture_data = Vec::<u8>::with_capacity(64);
        // for _index in 0..16 {
        //     texture_data.push(0);
        //     texture_data.push(0);
        //     texture_data.push(0);
        //     texture_data.push(255);
        // }

        // let upload_default = UploadTextureJobs {
        //     texture: default_black_texture,
        //     texture_data: vec![texture_data],
        // };

        Self {
            block_size,
            device_context: device_context.clone(),
            texture_info: Vec::new(),
            upload_jobs: Vec::new(),
            index_allocator,
        }
    }

    pub fn allocate_texture(
        &mut self,
        texture_def: &TextureDef,
        texture_data: &TextureData,
    ) -> GpuTextureId {
        let index = self.index_allocator.acquire_index();
        let new_texture = self.device_context.create_texture(texture_def);
        let texture_view_def = TextureViewDef::as_shader_resource_view(texture_def);
        // self.bindless_array[bindless_id as usize] = new_texture.create_view(&texture_view_def);

        if index as usize >= self.texture_info.len() {
            let required_size = next_multiple_of(index as usize, self.block_size as usize);
            self.texture_info
                .resize(required_size, TextureInfo::default());
        }
        let mut texture_info = &mut self.texture_info[index as usize];
        let generation = texture_info.generation;
        texture_info.state = TextureState::QueuedForUpload;
        texture_info.texture_view = Some(new_texture.create_view(&texture_view_def));

        let gpu_texture_id = GpuTextureId { generation, index };

        self.upload_jobs.push(UploadTextureJob {
            gpu_texture_id,
            texture_data: Some(texture_data.clone()),
        });

        gpu_texture_id
    }

    pub fn release_texture(&mut self, gpu_texture_id: GpuTextureId) {
        unimplemented!();
    }

    pub fn update_texture(&mut self, gpu_texture_id: GpuTextureId, texture_def: &TextureDef) {
        unimplemented!();

        // let bindless_id = self.index_allocator.acquire_index();

        // if let Some(stored_id) = self.ref_to_gpu_id.get_mut(&texture_component.texture_id) {
        //     *stored_id = bindless_id;
        // } else {
        //     self.ref_to_gpu_id
        //         .insert(texture_component.texture_id, bindless_id);
        // }

        // let format = match texture_component.format {
        //     TextureFormat::BC1 => Format::BC1_RGBA_UNORM_BLOCK,
        //     TextureFormat::BC3 => Format::BC3_UNORM_BLOCK,
        //     TextureFormat::BC4 => Format::BC4_UNORM_BLOCK,
        //     TextureFormat::BC7 => Format::BC7_UNORM_BLOCK,
        // };

        // let texture_def = TextureDef {
        //     extents: Extents3D {
        //         width: texture_component.width,
        //         height: texture_component.height,
        //         depth: 1,
        //     },
        //     array_length: 1,
        //     mip_count: texture_component.texture_data.len() as u32,
        //     format,
        //     usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
        //     resource_flags: ResourceFlags::empty(),
        //     mem_usage: MemoryUsage::GpuOnly,
        //     tiling: TextureTiling::Optimal,
        // };

        // let new_texture = self.device_context.create_texture(&texture_def);

        // let texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        // self.bindless_array[bindless_id as usize] = new_texture.create_view(&texture_view_def);
        // // self.textures.push(new_texture);

        // // self.upload_jobs.push(UploadTextureJobs {
        // //     texture: new_texture,
        // //     texture_data: std::mem::take(&mut texture_component.texture_data),
        // // });
    }

    // pub fn allocate_bindless_id(&mut self, texture_id: ResourceTypeAndId) -> u32 {
    //     // let bindless_id = if let Some(stored_id) = self.ref_to_gpu_id.get(&texture_id) {
    //     //     *stored_id
    //     // } else {
    //     //     let bindless_id = self.index_allocator.acquire_index();
    //     //     self.ref_to_gpu_id.insert(texture_id, bindless_id);
    //     //     bindless_id
    //     // };
    //     // bindless_id
    //     unimplemented!()
    // }
    #[span_fn]
    #[allow(clippy::needless_pass_by_value)]
    pub fn upload_textures(
        &mut self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
    ) {
        let mut upload_jobs = std::mem::take(&mut self.upload_jobs);

        for upload_job in &upload_jobs {
            let gpu_texture_id = upload_job.gpu_texture_id;
            if self.is_valid(gpu_texture_id) {
                let texture_info = self.texture_info_mut(gpu_texture_id);
                texture_info.state = TextureState::Uploaded;
                let texture = self.texture_handle(gpu_texture_id);
                let texture_data = upload_job.texture_data.as_ref().unwrap();
                let mip_slices = texture_data.data();
                for (mip_level, mip_data) in mip_slices.iter().enumerate() {
                    upload_texture_data(
                        device_context,
                        cmd_buffer,
                        texture,
                        mip_data,
                        mip_level as u8,
                    );
                }
            }
        }

        upload_jobs.resize(0, UploadTextureJob::default());

        self.upload_jobs = upload_jobs;
    }

    fn update_persistent_descriptor_set(
        &mut self,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) {
        for (index, texture_info) in self.texture_info.iter_mut().enumerate() {
            if texture_info.state == TextureState::Uploaded {
                persistent_descriptor_set_manager
                    .set_texture_(index as u32, texture_info.texture_view.as_ref().unwrap());
                texture_info.state = TextureState::Valid;
            }
        }
    }

    pub fn is_valid(&self, gpu_texture_id: GpuTextureId) -> bool {
        let index = gpu_texture_id.index as usize;
        if index >= self.texture_info.len() {
            return false;
        }
        let texture_info = &self.texture_info[index];
        let generation = gpu_texture_id.generation;
        return texture_info.generation == generation;
    }

    fn texture_handle(&self, gpu_texture_id: GpuTextureId) -> &Texture {
        assert!(self.is_valid(gpu_texture_id));
        let texture_info = self.texture_info(gpu_texture_id);
        texture_info.texture_view.as_ref().unwrap().texture()
    }

    fn texture_info(&self, gpu_texture_id: GpuTextureId) -> &TextureInfo {
        assert!(self.is_valid(gpu_texture_id));
        &self.texture_info[gpu_texture_id.index as usize]
    }

    fn texture_info_mut(&mut self, gpu_texture_id: GpuTextureId) -> &mut TextureInfo {
        assert!(self.is_valid(gpu_texture_id));
        &mut self.texture_info[gpu_texture_id.index as usize]
    }
}

pub struct TextureResourceManager {
    ref_to_gpu_id: BTreeMap<ResourceTypeAndId, u32>,
    gpu_texture_ids: [GpuTextureId; MaterialTextureType::COUNT],
    // bindless_texture_ids: [u32; MaterialTextureType::COUNT],
}

impl TextureResourceManager {
    pub fn new(texture_manager: &mut TextureManager) -> Self {
        let gpu_texture_ids: [GpuTextureId; MaterialTextureType::COUNT] =
            MaterialTextureType::iter()
                .map(|mat_tex_type| match mat_tex_type {
                    MaterialTextureType::Albedo
                    | MaterialTextureType::Normal
                    | MaterialTextureType::Roughness
                    | MaterialTextureType::Metalness => {
                        // todo: implement data variation
                        let texture_def = TextureDef {
                            extents: Extents3D {
                                width: 4,
                                height: 4,
                                depth: 1,
                            },
                            array_length: 1,
                            mip_count: 1,
                            format: Format::R8G8B8A8_UNORM,
                            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                                | ResourceUsage::AS_TRANSFERABLE,
                            resource_flags: ResourceFlags::empty(),
                            mem_usage: MemoryUsage::GpuOnly,
                            tiling: TextureTiling::Linear,
                        };

                        let mut texture_data = Vec::<u8>::with_capacity(64);
                        for _index in 0..16 {
                            texture_data.push(0);
                            texture_data.push(0);
                            texture_data.push(0);
                            texture_data.push(255);
                        }

                        texture_manager
                            .allocate_texture(&texture_def, &TextureData::from_slice(&texture_data))
                    }
                })
                .collect::<Vec<GpuTextureId>>()
                .as_slice()
                .try_into()
                .unwrap();

        Self {
            ref_to_gpu_id: BTreeMap::new(),
            gpu_texture_ids,
        }
    }

    pub fn bindless_index_for_texture_resource(
        &self,
        texture_type: MaterialTextureType,
        optional_id: &Option<TextureReferenceType>,
    ) -> u32 {
        if let Some(texture_id) = optional_id {
            if let Some(id) = self.ref_to_gpu_id.get(&texture_id.id()) {
                *id
            } else {
                self.gpu_texture_ids[texture_type as usize].index
            }
        } else {
            u32::MAX
        }
    }
}

fn next_multiple_of(value: usize, multiple: usize) -> usize {
    ((value + multiple - 1) / multiple) * multiple
}

fn upload_texture_data(
    device_context: &DeviceContext,
    cmd_buffer: &HLCommandBuffer<'_>,
    texture: &Texture,
    data: &[u8],
    mip_level: u8,
) {
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

    cmd_buffer.resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::UNDEFINED,
            ResourceState::COPY_DST,
        )],
    );

    cmd_buffer.copy_buffer_to_texture(
        &staging_buffer,
        texture,
        &CmdCopyBufferToTextureParams {
            mip_level,
            ..CmdCopyBufferToTextureParams::default()
        },
    );

    cmd_buffer.resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::COPY_DST,
            ResourceState::SHADER_RESOURCE,
        )],
    );
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_startup(
    mut commands: Commands<'_, '_>,
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
) {
    // let texture_def = TextureDef {
    //     extents: Extents3D {
    //         width: 4,
    //         height: 4,
    //         depth: 1,
    //     },
    //     array_length: 1,
    //     mip_count: 1,
    //     format: Format::R8G8B8A8_UNORM,
    //     usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
    //     resource_flags: ResourceFlags::empty(),
    //     mem_usage: MemoryUsage::GpuOnly,
    //     tiling: TextureTiling::Linear,
    // };

    // let mut texture_data = Vec::<u8>::with_capacity(64);
    // for _index in 0..16 {
    //     texture_data.push(0);
    //     texture_data.push(0);
    //     texture_data.push(0);
    //     texture_data.push(255);
    // }

    // texture_manager.allocate_texture(&texture_def, &TextureData::from_slice(&texture_data));;
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_added(
    mut commands: Commands<'_, '_>,
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
    q_added_textures: Query<'_, '_, (Entity, &TextureComponent), Added<TextureComponent>>,
) {
    for (entity, texture_component) in q_added_textures.iter() {
        let format = match texture_component.format {
            TextureFormat::BC1 => Format::BC1_RGBA_UNORM_BLOCK,
            TextureFormat::BC3 => Format::BC3_UNORM_BLOCK,
            TextureFormat::BC4 => Format::BC4_UNORM_BLOCK,
            TextureFormat::BC7 => Format::BC7_UNORM_BLOCK,
        };

        let texture_def = TextureDef {
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
        };

        let gpu_texture_id =
            texture_manager.allocate_texture(&texture_def, &texture_component.texture_data);

        // let bindless_texture_id =
        //     texture_manager.allocate_bindless_id(texture_component.texture_id);

        commands
            .entity(entity)
            .insert(GPUTextureComponent { gpu_texture_id });
        // .insert(BindlessTextureComponent {
        //     bindless_texture_id,
        // });
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_modified(
    renderer: Res<'_, Renderer>,
    mut texture_manager: ResMut<'_, TextureManager>,
    q_modified_textures: Query<
        '_,
        '_,
        (Entity, &TextureComponent, &GPUTextureComponent),
        Changed<TextureComponent>,
    >,
) {
    for (entity, texture_component, gpu_texture_component) in q_modified_textures.iter() {
        let format = match texture_component.format {
            TextureFormat::BC1 => Format::BC1_RGBA_UNORM_BLOCK,
            TextureFormat::BC3 => Format::BC3_UNORM_BLOCK,
            TextureFormat::BC4 => Format::BC4_UNORM_BLOCK,
            TextureFormat::BC7 => Format::BC7_UNORM_BLOCK,
        };

        let texture_def = TextureDef {
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
        };

        texture_manager.update_texture(gpu_texture_component.gpu_texture_id, &texture_def);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_removed(
    removed_entities: RemovedComponents<'_, TextureComponent>,
    mut texture_manager: ResMut<'_, TextureManager>,
) {
    for removed_entity in removed_entities.iter() {
        unimplemented!();
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn upload_textures(
    renderer: Res<'_, Renderer>,
    pipeline_manager: Res<'_, PipelineManager>,
    mut texture_manager: ResMut<'_, TextureManager>,
    descriptor_heap_manager: Res<'_, DescriptorHeapManager>,
    // q_modified_gpu_textures: Query<
    //     '_,
    //     '_,
    //     (Entity, &TextureComponent, &GPUTextureComponent),
    //     Changed<GPUTextureComponent>,
    // >,
) {
    // if !q_modified_gpu_textures.is_empty() {
    let render_context = RenderContext::new(&renderer, &descriptor_heap_manager, &pipeline_manager);
    let cmd_buffer = render_context.alloc_command_buffer();

    texture_manager.upload_textures(renderer.device_context(), &cmd_buffer);

    render_context
        .graphics_queue()
        .submit(&mut [cmd_buffer.finalize()], &[], &[], None);
    // }
}

fn update_persistent_descriptor_set(
    mut texture_manager: ResMut<'_, TextureManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    texture_manager.update_persistent_descriptor_set(&mut persistent_descriptor_set_manager);
}
