use std::collections::BTreeMap;

use lgn_app::{App, CoreStage, Plugin};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{Added, Changed, Entity, Query, Res, ResMut};
use lgn_graphics_api::{BufferView, VertexBufferBinding};
use lgn_math::Vec4;
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;

use crate::{
    cgen,
    components::{MaterialComponent, VisualComponent},
    labels::RenderStage,
    resources::SharedTextureId,
    Renderer,
};

use super::{
    BindlessTextureManager, IndexAllocator, UnifiedStaticBufferAllocator, UniformGPUData,
    UniformGPUDataUpdater,
};

#[derive(Default)]
pub struct GpuDataPlugin {}

pub(crate) struct GpuDataManager<K, T> {
    gpu_data: UniformGPUData<T>,
    index_allocator: IndexAllocator,
    data_map: BTreeMap<K, Vec<(u32, u64)>>,
    default_uploaded: bool,
    default_id: u32,
    default_va: u64,
}

impl<K, T> GpuDataManager<K, T> {
    pub fn new(page_size: u64, block_size: u32) -> Self {
        let mut index_allocator = IndexAllocator::new(block_size);
        let gpu_data = UniformGPUData::<T>::new(None, page_size);

        Self {
            gpu_data,
            index_allocator,
            data_map: BTreeMap::new(),
            default_uploaded: false,
            default_id: u32::MAX,
            default_va: u64::MAX,
        }
    }

    pub fn alloc_gpu_data(
        &mut self,
        key: K,
        allocator: &UnifiedStaticBufferAllocator,
        
    ) -> (u32, u64)
    where
        K: Ord,
    {
        let gpu_data_id = self.index_allocator.acquire_index();
        let gpu_data_va = self.gpu_data.ensure_index_allocated(allocator, gpu_data_id);

        if let Some(gpu_data) = self.data_map.get_mut(&key) {
            gpu_data.push((gpu_data_id, gpu_data_va));
        } else {
            self.data_map.insert(key, vec![(gpu_data_id, gpu_data_va)]);
        }
        (gpu_data_id, gpu_data_va)
    }

    pub fn id_for_index(&self, optional: Option<K>, index: usize) -> u32
    where
        K: Ord,
    {
        if let Some(key) = optional {
            if let Some(value) = self.data_map.get(&key) {
                return value[index].0;
            }
        }
        self.default_id
    }

    pub fn va_for_index(&self, optional: Option<K>, index: usize) -> u64
    where
        K: Ord,
    {
        if let Some(key) = optional {
            if let Some(value) = self.data_map.get(&key) {
                return value[index].1;
            }
        }
        self.default_va
    }

    pub fn update_gpu_data(
        &self,
        key: &K,
        dest_idx: usize,
        data: &[T],
        updater: &mut UniformGPUDataUpdater,
    ) where
        K: Ord,
    {
        if let Some(gpu_data) = self.data_map.get(key) {
            updater.add_update_jobs(data, gpu_data[dest_idx].1);
        }
    }

    pub fn remove_gpu_data(&mut self, key: &K) -> Option<Vec<u32>>
    where
        K: Ord,
    {
        if let Some(gpu_data) = self.data_map.remove(key) {
            let mut instance_ids = Vec::with_capacity(gpu_data.len());
            for data in gpu_data {
                instance_ids.push(data.0);
            }
            self.index_allocator.release_index_ids(&instance_ids);

            Some(instance_ids)
        } else {
            None
        }
    }

    pub fn upload_default(
        &mut self,
        default: T,
        allocator: &UnifiedStaticBufferAllocator,
        updater: &mut UniformGPUDataUpdater,
    ) {
        if !self.default_uploaded {
            let mut index_block = None;
            self.default_id = self.index_allocator.acquire_index(&mut index_block);
            self.default_va = self
                .gpu_data
                .ensure_index_allocated(allocator, self.default_id);
            self.index_allocator
                .release_index_block(index_block.unwrap());

            updater.add_update_jobs(&[default], self.default_va);
        }
    }

    pub fn default_uploaded(&mut self) {
        self.default_uploaded = true;
    }
}

pub(crate) type GpuEntityTransformManager =
    GpuDataManager<Entity, cgen::cgen_type::GpuInstanceTransform>;
pub(crate) type GpuEntityColorManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuPickingDataManager =
    GpuDataManager<Entity, cgen::cgen_type::GpuInstancePickingData>;
pub(crate) type GpuMaterialManager =
    GpuDataManager<ResourceTypeAndId, cgen::cgen_type::MaterialData>;

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        //
        // Resources
        //
        app.insert_resource(GpuEntityTransformManager::new(64 * 1024, 1024));
        app.insert_resource(GpuEntityColorManager::new(64 * 1024, 256));
        app.insert_resource(GpuPickingDataManager::new(64 * 1024, 1024));

        app.insert_resource(GpuMaterialManager::new(64 * 1024, 256));

        //
        // Stage PostUpdate
        //
        app.add_system_to_stage(CoreStage::PostUpdate, alloc_color_address);
        app.add_system_to_stage(CoreStage::PostUpdate, alloc_transform_address);
        app.add_system_to_stage(CoreStage::PostUpdate, alloc_material_address);
        app.add_system_to_stage(CoreStage::PostUpdate, upload_default_material);

        //
        // Stage Prepare
        //
        app.add_system_to_stage(RenderStage::Prepare, upload_transform_data);
        app.add_system_to_stage(RenderStage::Prepare, upload_material_data);

        //
        // Stage: Render
        //
        app.add_system_to_stage(RenderStage::Render, mark_defaults_as_uploaded);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn alloc_color_address(
    renderer: Res<'_, Renderer>,
    mut color_manager: ResMut<'_, GpuEntityColorManager>,
    query: Query<'_, '_, Entity, Added<VisualComponent>>,
) {
    for entity in query.iter() {
        color_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn alloc_transform_address(
    renderer: Res<'_, Renderer>,
    mut transform_manager: ResMut<'_, GpuEntityTransformManager>,
    query: Query<'_, '_, Entity, Added<GlobalTransform>>,
) {
    for entity in query.iter() {
        transform_manager.alloc_gpu_data(
            entity,
            renderer.static_buffer_allocator(),
            
        );
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn alloc_material_address(
    renderer: Res<'_, Renderer>,
    mut material_manager: ResMut<'_, GpuMaterialManager>,
    query: Query<'_, '_, &MaterialComponent, Added<MaterialComponent>>,
) {
    for material in query.iter() {
        material_manager.alloc_gpu_data(
            material.material_id,
            renderer.static_buffer_allocator(),

        );
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn upload_transform_data(
    renderer: Res<'_, Renderer>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
    query: Query<'_, '_, (Entity, &GlobalTransform), Changed<GlobalTransform>>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for (entity, transform) in query.iter() {
        let mut world = cgen::cgen_type::GpuInstanceTransform::default();
        world.set_world(transform.compute_matrix().into());

        transform_manager.update_gpu_data(&entity, 0, &[world], &mut updater);
    }

    renderer.add_update_job_block(updater.job_blocks());
}

#[allow(clippy::needless_pass_by_value)]
fn upload_default_material(
    renderer: Res<'_, Renderer>,
    texture_manager: Res<'_, TextureManager>,
    texture_resource_manager: Res<'_, TextureResourceManager>,
	 shared_resources_manager: Res<'_, SharedResourcesManager>,
    mut material_manager: ResMut<'_, GpuMaterialManager>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    let mut default_material = cgen::cgen_type::MaterialData::default();
    default_material.set_base_albedo(Vec4::new(0.8, 0.8, 0.8, 1.0).into());
    default_material.set_base_metalness(0.0.into());
    default_material.set_reflectance(0.5.into());
    default_material.set_base_roughness(0.4.into());
    default_material.set_albedo_texture(u32::MAX.into());
    default_material.set_normal_texture(u32::MAX.into());
    default_material.set_metalness_texture(u32::MAX.into());
    default_material.set_roughness_texture(u32::MAX.into());

    material_manager.upload_default(
        default_material,
        renderer.static_buffer_allocator(),
        &mut updater,
    );

    renderer.add_update_job_block(updater.job_blocks());
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn upload_material_data(
    renderer: Res<'_, Renderer>,
    material_manager: ResMut<'_, GpuMaterialManager>,
    bindless_textures: ResMut<'_, BindlessTextureManager>,
    query: Query<'_, '_, &MaterialComponent, Changed<MaterialComponent>>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for material in query.iter() {
        let mut gpu_material = cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(material.base_albedo.r) / 255.0f32,
            f32::from(material.base_albedo.g) / 255.0f32,
            f32::from(material.base_albedo.b) / 255.0f32,
            f32::from(material.base_albedo.a) / 255.0f32,
        );
        gpu_material.set_base_albedo(color.into());
        gpu_material.set_base_metalness(material.base_metalness.into());
        gpu_material.set_reflectance(material.reflectance.into());
        gpu_material.set_base_roughness(material.base_roughness.into());
        gpu_material.set_albedo_texture(
            material
                .albedo_texture
                .as_ref()
                .map(|texture_id| {
            texture_resource_manager
                        .bindless_index_for_resource_id(&texture_manager, texture_id)
                        .unwrap_or_else(|| {
                            shared_resources_manager
                                .default_texture_bindless_index(SharedTextureId::Albedo)
                        })
                })
                .unwrap()
                .into(),
        );
        gpu_material.set_normal_texture(
            material
                .normal_texture
                .as_ref()
                .map(|texture_id| {
            texture_resource_manager
                        .bindless_index_for_resource_id(&texture_manager, texture_id)
                        .unwrap_or_else(|| {
                            shared_resources_manager
                                .default_texture_bindless_index(SharedTextureId::Normal)
                        })
                })
                .unwrap()
                .into(),
        );
        gpu_material.set_metalness_texture(
            material
                .metalness_texture
                .as_ref()
                .map(|texture_id| {
            texture_resource_manager
                        .bindless_index_for_resource_id(&texture_manager, texture_id)
                        .unwrap_or_else(|| {
                            shared_resources_manager
                                .default_texture_bindless_index(SharedTextureId::Metalness)
                        })
                })
                .unwrap()
                .into(),
        );
        gpu_material.set_roughness_texture(
            material
                .roughness_texture
                .as_ref()
                .map(|texture_id| {
            texture_resource_manager
                        .bindless_index_for_resource_id(&texture_manager, texture_id)
                        .unwrap_or_else(|| {
                            shared_resources_manager
                                .default_texture_bindless_index(SharedTextureId::Roughness)
                        })
                })
                .unwrap()
                .into(),
        );

        material_manager.update_gpu_data(&material.material_id, 0, &[gpu_material], &mut updater);
    }

    renderer.add_update_job_block(updater.job_blocks());
}

#[span_fn]
fn mark_defaults_as_uploaded(mut material_manager: ResMut<'_, GpuMaterialManager>) {
    material_manager.default_uploaded();
}

pub(crate) struct GpuVaTableForGpuInstance {
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
        gpu_instance: u32,
        va_table_address: u32,
    ) {
        let offset_for_gpu_instance = self.static_allocation.offset() + u64::from(gpu_instance) * 4;

        updater.add_update_jobs(&[va_table_address], offset_for_gpu_instance);
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.static_allocation.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.static_allocation
            .structured_buffer_view(struct_size, read_only)
    }
}
