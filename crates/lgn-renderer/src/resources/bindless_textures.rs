use std::collections::BTreeMap;

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
    components::TextureComponent, hl_gfx_api::HLCommandBuffer, labels::RenderStage, RenderContext,
    Renderer,
};

use super::{DescriptorHeapManager, IndexAllocator, PipelineManager};

struct UploadTextureJobs {
    texture: Texture,
    texture_data: Vec<Vec<u8>>,
}

#[derive(Component)]
struct GPUTextureComponent {
    texture: Texture,
}

#[derive(Component)]
struct BindlessTextureComponent {
    bindless_id: u32,
}

pub struct BindlessTexturePlugin {
    device_context: DeviceContext,
}

impl BindlessTexturePlugin {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
        }
    }
}

impl Plugin for BindlessTexturePlugin {
    fn build(&self, app: &mut lgn_app::App) {
        app.insert_resource(BindlessTextureManager::new(&self.device_context, 256));
        app.add_startup_system(on_startup);
        app.add_system_to_stage(CoreStage::PostUpdate, on_texture_add_or_modify);
        app.add_system_to_stage(CoreStage::PostUpdate, on_texture_removed);
        app.add_system_to_stage(RenderStage::Prepare, upload_bindless_textures);
    }
}

pub struct BindlessTextureManager {
    // textures: Vec<Texture>,
    bindless_array: Vec<TextureView>,
    upload_jobs: Vec<UploadTextureJobs>,
    index_allocator: IndexAllocator,
    ref_to_gpu_id: BTreeMap<ResourceTypeAndId, u32>,
    entity_map: BTreeMap<Entity, u32>,
    default_texture_id: u32,
}

impl BindlessTextureManager {
    pub fn new(device_context: &DeviceContext, array_size: usize) -> Self {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut index_allocator = IndexAllocator::new(256);

        let default_texture_id = index_allocator.acquire_index();

        let default_black_texture = device_context.create_texture(&texture_def);

        let default_black_texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        let descriptor_array =
            vec![default_black_texture.create_view(&default_black_texture_view_def,); array_size];

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        let upload_default = UploadTextureJobs {
            texture: default_black_texture,
            texture_data: vec![texture_data],
        };

        Self {
            // textures: vec![default_black_texture],
            bindless_array: descriptor_array,
            upload_jobs: vec![upload_default],
            index_allocator,
            ref_to_gpu_id: BTreeMap::new(),
            entity_map: BTreeMap::new(),
            default_texture_id,
        }
    }

    pub fn allocate_texture(
        &mut self,
        device_context: &DeviceContext,
        texture_component: &TextureComponent,
    ) -> (Texture, u32) {
        let bindless_id = self.index_allocator.acquire_index();

        if let Some(stored_id) = self.ref_to_gpu_id.get_mut(&texture_component.texture_id) {
            *stored_id = bindless_id;
        } else {
            self.ref_to_gpu_id
                .insert(texture_component.texture_id, bindless_id);
        }

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
            mip_count: texture_component.texture_data.len() as u32,
            format,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let new_texture = device_context.create_texture(&texture_def);

        let texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        self.bindless_array[bindless_id as usize] = new_texture.create_view(&texture_view_def);
        // self.textures.push(new_texture);

        // self.upload_jobs.push(UploadTextureJobs {
        //     texture: new_texture,
        //     texture_data: std::mem::take(&mut texture_component.texture_data),
        // });

        (new_texture, bindless_id)
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let binding_id = self.entity_map.get(&entity);
        let binding_id = binding_id.unwrap();
    }

    #[span_fn]
    #[allow(clippy::needless_pass_by_value)]
    pub fn upload_textures(
        &mut self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
    ) {
        for upload in self.upload_jobs.drain(..) {
            for mip_level in 0..upload.texture_data.len() as u8 {
                upload_texture_data(
                    device_context,
                    cmd_buffer,
                    &upload.texture,
                    &upload.texture_data[mip_level as usize],
                    mip_level,
                );
            }
        }
    }

    pub fn bindless_id_for_texture(&self, optional_id: &Option<TextureReferenceType>) -> u32 {
        if let Some(texture_id) = optional_id {
            if let Some(id) = self.ref_to_gpu_id.get(&texture_id.id()) {
                *id
            } else {
                self.default_texture_id
            }
        } else {
            u32::MAX
        }
    }

    pub fn default_black_texture_view(&self) -> TextureView {
        self.bindless_array[0].clone()
    }

    pub fn bindless_texures_for_update(&self) -> Vec<TextureView> {
        self.bindless_array.clone()
    }
}

pub fn upload_texture_data(
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

    mut bindless_tex_manager: ResMut<'_, BindlessTextureManager>,    
) {
        
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_add_or_modify(
    mut commands: Commands<'_, '_>,
    renderer: Res<'_, Renderer>,

    mut bindless_tex_manager: ResMut<'_, BindlessTextureManager>,

    q_added_textures: Query<'_, '_, (Entity, &TextureComponent), Added<TextureComponent>>,
    q_modified_textures: Query<'_, '_, (Entity, &TextureComponent), Changed<TextureComponent>>,
) {
    for (entity, texture_component) in q_added_textures.iter() {
        let (texture, bindless_id) =
            bindless_tex_manager.allocate_texture(renderer.device_context(), texture_component);

        commands
            .entity(entity)
            .insert(GPUTextureComponent { texture })
            .insert(BindlessTextureComponent { bindless_id });
    }

    for (entity, texture_component) in q_modified_textures.iter() {
        let (texture, _) =
            bindless_tex_manager.allocate_texture(renderer.device_context(), texture_component);

        commands
            .entity(entity)
            .insert(GPUTextureComponent { texture });
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_removed(
    removed_entities: RemovedComponents<'_, TextureComponent>,
    mut bindless_tex_manager: ResMut<'_, BindlessTextureManager>,
) {
    for entity in removed_entities.iter() {
        bindless_tex_manager.remove_entity(entity);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn upload_bindless_textures(
    renderer: Res<'_, Renderer>,
    pipeline_manager: Res<'_, PipelineManager>,
    mut bindless_tex_manager: ResMut<'_, BindlessTextureManager>,
    descriptor_heap_manager: Res<'_, DescriptorHeapManager>,
    q_modified_gpu_textures: Query<
        '_,
        '_,
        (Entity, &TextureComponent, &GPUTextureComponent),
        Changed<GPUTextureComponent>,
    >,
) {
    if !q_modified_gpu_textures.is_empty() {
        let render_context =
            RenderContext::new(&renderer, &descriptor_heap_manager, &pipeline_manager);
        let cmd_buffer = render_context.alloc_command_buffer();

        for (a, b, c) in q_modified_gpu_textures.iter() {
            dbg!(&a);
        }

        bindless_tex_manager.upload_textures(renderer.device_context(), &cmd_buffer);

        render_context
            .graphics_queue()
            .submit(&mut [cmd_buffer.finalize()], &[], &[], None);
    }
}
