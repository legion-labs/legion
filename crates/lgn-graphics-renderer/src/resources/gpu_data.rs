use std::{collections::BTreeMap, sync::Arc};

use lgn_app::{App, Plugin};
use lgn_data_runtime::{AssetRegistry, AssetRegistryEvent, Resource, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_graphics_api::{BufferView, VertexBufferBinding};
use lgn_graphics_data::runtime_texture::TextureReferenceType;
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
    IndexAllocator, SharedResourcesManager, StaticBufferAllocation, TextureEvent, TextureManager,
    UnifiedStaticBufferAllocator, UniformGPUData, UniformGPUDataUpdater,
};

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum GpuDataPluginLabel {
    UpdateDone,
}

pub use lgn_graphics_data::runtime::Material as MaterialAsset;

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
        let index_allocator = IndexAllocator::new(block_size);
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

    pub fn alloc_gpu_data(&mut self, key: K, allocator: &UnifiedStaticBufferAllocator) -> (u32, u64)
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
        data: &T,
        updater: &mut UniformGPUDataUpdater,
    ) where
        K: Ord,
    {
        if let Some(gpu_data) = self.data_map.get(key) {
            let data_slice = std::slice::from_ref(data);
            updater.add_update_jobs(data_slice, gpu_data[dest_idx].1);
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
            self.default_id = self.index_allocator.acquire_index();
            self.default_va = self
                .gpu_data
                .ensure_index_allocated(allocator, self.default_id);

            updater.add_update_jobs(&[default], self.default_va);
            self.default_uploaded = true;
        }
    }
}

#[derive(Default, Component)]
struct GPUMaterialComponent;

pub(crate) type GpuEntityTransformManager = GpuDataManager<Entity, cgen::cgen_type::Transform>;
pub(crate) type GpuEntityColorManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuPickingDataManager =
    GpuDataManager<Entity, cgen::cgen_type::GpuInstancePickingData>;
pub(crate) type GpuMaterialData = GpuDataManager<ResourceTypeAndId, cgen::cgen_type::MaterialData>;

struct UploadMaterialJob {
    resource_id: ResourceTypeAndId,
    material_data: cgen::cgen_type::MaterialData,
}

pub struct MaterialManager {
    material_to_texture_ids: BTreeMap<ResourceTypeAndId, Vec<ResourceTypeAndId>>,
    upload_queue: Vec<UploadMaterialJob>,
    gpu_material_data: GpuMaterialData,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            material_to_texture_ids: BTreeMap::new(),
            upload_queue: Vec::new(),
            gpu_material_data: GpuMaterialData::new(64 * 1024, 256),
        }
    }

    // todo: no real reason to not make that public
    pub(crate) fn gpu_data(&self) -> &GpuMaterialData {
        &self.gpu_material_data
    }

    pub(crate) fn gpu_data_mut(&mut self) -> &mut GpuMaterialData {
        &mut self.gpu_material_data
    }

    pub fn add_material(
        &mut self,
        resource_id: ResourceTypeAndId,
        material_asset: &MaterialAsset,
        allocator: &UnifiedStaticBufferAllocator,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        self.gpu_material_data
            .alloc_gpu_data(resource_id, allocator);

        let job = UploadMaterialJob {
            resource_id,
            material_data: Self::material_asset_to_material_data(
                material_asset,
                texture_manager,
                shared_resources_manager,
            ),
        };
        self.upload_queue.push(job);

        self.material_to_texture_ids.insert(
            resource_id,
            Self::collect_texture_dependencies(material_asset),
        );
    }

    pub fn change_material(
        &mut self,
        resource_id: ResourceTypeAndId,
        material_asset: &MaterialAsset,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        // TODO(vdbdd): not tested
        let job = UploadMaterialJob {
            resource_id,
            material_data: Self::material_asset_to_material_data(
                material_asset,
                texture_manager,
                shared_resources_manager,
            ),
        };
        self.upload_queue.push(job);
        self.material_to_texture_ids.insert(
            resource_id,
            Self::collect_texture_dependencies(material_asset),
        );
    }

    pub fn remove_material(&mut self, resource_id: ResourceTypeAndId) {
        // TODO(vdbdd): not tested
        self.material_to_texture_ids.remove(&resource_id);
        self.gpu_material_data.remove_gpu_data(&resource_id);
    }

    pub fn on_texture_state_changed(
        &mut self,
        texture_id: &ResourceTypeAndId,
        asset_registry: &AssetRegistry,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        // todo: can be optimized by having a map ( texture_id -> material )
        for (material_id, texture_ids) in &self.material_to_texture_ids {
            if texture_ids.contains(texture_id) {
                if let Some(material_asset) = asset_registry
                    .get_untyped(*material_id)
                    .and_then(|h| h.get::<MaterialAsset>(asset_registry))
                {
                    let job = UploadMaterialJob {
                        resource_id: *material_id,
                        material_data: Self::material_asset_to_material_data(
                            &material_asset,
                            texture_manager,
                            shared_resources_manager,
                        ),
                    };
                    self.upload_queue.push(job);
                }
            }
        }
    }

    fn collect_texture_dependencies(material_asset: &MaterialAsset) -> Vec<ResourceTypeAndId> {
        let mut result = Vec::new();

        if let Some(texture) = &material_asset.albedo {
            result.push(texture.id());
        }
        if let Some(texture) = &material_asset.normal {
            result.push(texture.id());
        }
        if let Some(texture) = &material_asset.metalness {
            result.push(texture.id());
        }
        if let Some(texture) = &material_asset.roughness {
            result.push(texture.id());
        }
        result
    }

    fn material_asset_to_material_data(
        material_asset: &MaterialAsset,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) -> cgen::cgen_type::MaterialData {
        let mut material_data = cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(material_asset.base_albedo.r) / 255.0f32,
            f32::from(material_asset.base_albedo.g) / 255.0f32,
            f32::from(material_asset.base_albedo.b) / 255.0f32,
            f32::from(material_asset.base_albedo.a) / 255.0f32,
        );
        material_data.set_base_albedo(color.into());
        material_data.set_base_metalness(material_asset.base_metalness.into());
        material_data.set_reflectance(material_asset.reflectance.into());
        material_data.set_base_roughness(material_asset.base_roughness.into());
        material_data.set_albedo_texture(
            Self::get_bindless_index(
                material_asset.albedo.as_ref(),
                SharedTextureId::Albedo,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_normal_texture(
            Self::get_bindless_index(
                material_asset.normal.as_ref(),
                SharedTextureId::Normal,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_metalness_texture(
            Self::get_bindless_index(
                material_asset.metalness.as_ref(),
                SharedTextureId::Metalness,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_roughness_texture(
            Self::get_bindless_index(
                material_asset.roughness.as_ref(),
                SharedTextureId::Roughness,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );

        material_data
    }

    fn get_bindless_index(
        texture_id: Option<&TextureReferenceType>,
        default_shared_id: SharedTextureId,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) -> u32 {
        if let Some(texture_id) = texture_id {
            texture_manager
                .bindless_index_for_resource_id(&texture_id.id())
                .unwrap_or_else(|| {
                    shared_resources_manager.default_texture_bindless_index(default_shared_id)
                })
        } else {
            shared_resources_manager.default_texture_bindless_index(default_shared_id)
        }
    }
}

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        //
        // Resources
        //
        app.insert_resource(GpuEntityTransformManager::new(64 * 1024, 1024));
        app.insert_resource(GpuEntityColorManager::new(64 * 1024, 256));
        app.insert_resource(GpuPickingDataManager::new(64 * 1024, 1024));
        app.insert_resource(MaterialManager::new());

        //
        // Stage Prepare
        //
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(alloc_color_address)
                .with_system(alloc_transform_address)
                .with_system(on_material_events)
                .with_system(on_texture_event)
                .with_system(upload_default_material)
                .label(GpuDataPluginLabel::UpdateDone),
        );

        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(upload_transform_data)
                .with_system(upload_material_data)
                .after(GpuDataPluginLabel::UpdateDone),
        );
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
        transform_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_material_events(
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    mut asset_loaded_events: EventReader<'_, '_, AssetRegistryEvent>,
    mut material_manager: ResMut<'_, MaterialManager>,
    renderer: Res<'_, Renderer>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
) {
    for asset_loaded_event in asset_loaded_events.iter() {
        match asset_loaded_event {
            AssetRegistryEvent::AssetLoaded(resource_id)
                if resource_id.kind == MaterialAsset::TYPE =>
            {
                if let Some(material_asset) = asset_registry
                    .get_untyped(*resource_id)
                    .and_then(|handle| handle.get::<MaterialAsset>(&asset_registry))
                {
                    material_manager.add_material(
                        *resource_id,
                        &material_asset,
                        renderer.static_buffer_allocator(),
                        &texture_manager,
                        &shared_resources_manager,
                    );
                }
            }
            AssetRegistryEvent::AssetChanged(resource_id)
                if resource_id.kind == MaterialAsset::TYPE =>
            {
                if let Some(material_asset) = asset_registry
                    .get_untyped(*resource_id)
                    .and_then(|handle| handle.get::<MaterialAsset>(&asset_registry))
                {
                    material_manager.change_material(
                        *resource_id,
                        &material_asset,
                        &texture_manager,
                        &shared_resources_manager,
                    );
                }
            }
            AssetRegistryEvent::AssetUnloaded(resource_id)
                if resource_id.kind == MaterialAsset::TYPE =>
            {
                material_manager.remove_material(*resource_id);
            }

            _ => (),
        }
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_event(
    mut event_reader: EventReader<'_, '_, TextureEvent>,
    mut material_manager: ResMut<'_, MaterialManager>,
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
) {
    for event in event_reader.iter() {
        match event {
            TextureEvent::StateChanged(texture_id_list) => {
                for texture_id in texture_id_list {
                    material_manager.on_texture_state_changed(
                        texture_id,
                        &asset_registry,
                        &texture_manager,
                        &shared_resources_manager,
                    );
                }
            }
        }
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
        let mut world = cgen::cgen_type::Transform::default();
        world.set_translation(transform.translation.into());
        world.set_rotation(Vec4::from(transform.rotation).into());
        world.set_scale(transform.scale.into());

        transform_manager.update_gpu_data(&entity, 0, &world, &mut updater);
    }

    renderer.add_update_job_block(updater.job_blocks());
}

#[allow(clippy::needless_pass_by_value)]
fn upload_default_material(
    renderer: Res<'_, Renderer>,
    mut material_manager: ResMut<'_, MaterialManager>,
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

    material_manager.gpu_data_mut().upload_default(
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
    material_manager: Res<'_, MaterialManager>,
    _texture_manager: Res<'_, TextureManager>,
    _shared_resources_manager: Res<'_, SharedResourcesManager>,
    _query: Query<'_, '_, &MaterialComponent>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for upload_item in &material_manager.upload_queue {
        let material_data = &upload_item.material_data;

        material_manager.gpu_data().update_gpu_data(
            &upload_item.resource_id,
            0,
            material_data,
            &mut updater,
        );
    }

    renderer.add_update_job_block(updater.job_blocks());
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
