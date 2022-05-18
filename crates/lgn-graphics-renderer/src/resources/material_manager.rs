use std::collections::BTreeMap;

use lgn_app::App;
use lgn_data_runtime::{ResourceDescriptor, ResourceId, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_graphics_data::runtime_texture::TextureReferenceType;
use lgn_math::Vec4;
use lgn_utils::{memory::round_size_up_to_alignment_u32, HashSet};

use crate::{
    components::{MaterialComponent, MaterialData},
    labels::RenderStage,
    resources::SharedTextureId,
    Renderer, ResourceStageLabel,
};

use super::{
    GpuDataManager, IndexAllocator, MissingVisualTracker, PersistentDescriptorSetManager,
    SamplerManager, SharedResourcesManager, TextureEvent, TextureManager,
    UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
};

type GpuMaterialData = GpuDataManager<MaterialId, crate::cgen::cgen_type::MaterialData>;

#[derive(Default, Component)]
struct GPUMaterialComponent;

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum MaterialManagerLabel {
    UpdateDone,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(u32);

impl MaterialId {
    pub fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for MaterialId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct Material {
    resource_id: ResourceTypeAndId,
    material_data: MaterialData,
    va: u64,
}

#[derive(Clone)]
enum MaterialSlot {
    Empty,
    Occupied(Material),
}

impl Material {
    pub fn resource_id(&self) -> &ResourceTypeAndId {
        &self.resource_id
    }

    pub fn va(&self) -> u64 {
        self.va
    }
}

const MATERIAL_BLOCK_SIZE: u32 = 2048;

pub struct MaterialManager {
    allocator: IndexAllocator,
    materials: Vec<MaterialSlot>,
    resource_id_to_material_id: BTreeMap<ResourceTypeAndId, MaterialId>,
    entity_to_material_id: BTreeMap<Entity, MaterialId>,
    material_id_to_texture_ids: BTreeMap<MaterialId, Vec<ResourceTypeAndId>>,
    upload_queue: HashSet<MaterialId>,
    gpu_material_data: GpuMaterialData,
    default_resource_id: ResourceTypeAndId,
    default_material_id: MaterialId,
    default_uploaded: bool,
}

impl MaterialManager {
    pub fn new() -> Self {
        let mut allocator = IndexAllocator::new(MATERIAL_BLOCK_SIZE);

        // TODO(vdbdd): redundant and useless. remove asap.
        let default_resource_id = ResourceTypeAndId {
            kind: lgn_graphics_data::runtime::Material::TYPE,
            id: ResourceId::new(),
        };

        let default_material_id = allocator.acquire_index();

        Self {
            allocator,
            materials: Vec::new(),
            resource_id_to_material_id: BTreeMap::new(),
            entity_to_material_id: BTreeMap::new(),
            material_id_to_texture_ids: BTreeMap::new(),
            upload_queue: HashSet::new(),
            gpu_material_data: GpuMaterialData::new(MATERIAL_BLOCK_SIZE),
            default_resource_id,
            default_material_id: default_material_id.into(),
            default_uploaded: false,
        }
    }

    pub fn init_ecs(app: &mut App) {
        //
        // Stage Prepare
        //
        app.add_system_set_to_stage(
            RenderStage::Resource,
            SystemSet::new()
                .with_system(on_material_added)
                .with_system(on_material_changed)
                .with_system(on_material_removed)
                .with_system(on_texture_event)
                .with_system(upload_default_material)
                .label(MaterialManagerLabel::UpdateDone)
                .after(ResourceStageLabel::Texture),
        );
        app.add_system_set_to_stage(
            RenderStage::Resource,
            SystemSet::new()
                .with_system(upload_material_data)
                .label(ResourceStageLabel::Material)
                .after(MaterialManagerLabel::UpdateDone),
        );
    }

    pub fn get_default_material_id(&self) -> MaterialId {
        self.default_material_id
    }

    pub fn get_material_id_from_resource_id(
        &self,
        resource_id: &ResourceTypeAndId,
    ) -> Option<MaterialId> {
        self.resource_id_to_material_id.get(resource_id).copied()
    }

    pub fn get_material_id_from_resource_id_unchecked(
        &self,
        resource_id: &ResourceTypeAndId,
    ) -> MaterialId {
        *self.resource_id_to_material_id.get(resource_id).unwrap()
    }

    pub fn is_material_ready(&self, material_id: MaterialId) -> bool {
        let slot = &self.materials[material_id.index() as usize];
        match slot {
            MaterialSlot::Empty => panic!("Invalid material id"),
            MaterialSlot::Occupied(_) => true,
        }
    }

    pub fn get_material(&self, material_id: MaterialId) -> &Material {
        let slot = &self.materials[material_id.index() as usize];
        match slot {
            MaterialSlot::Empty => panic!("Invalid material id"),
            MaterialSlot::Occupied(material) => material,
        }
    }

    fn get_material_mut(&mut self, material_id: MaterialId) -> &mut Material {
        let slot = &mut self.materials[material_id.index() as usize];
        match slot {
            MaterialSlot::Empty => panic!("Invalid material id"),
            MaterialSlot::Occupied(material) => material,
        }
    }

    fn add_material(
        &mut self,
        entity: Entity,
        material_component: &MaterialComponent,
        allocator: &UnifiedStaticBufferAllocator,
    ) {
        let material_resource_id = material_component.resource.id();
        let material_id: MaterialId = self.allocator.acquire_index().into();
        let material_data = material_component.material_data.clone();

        self.alloc_material(material_id, material_resource_id, material_data, allocator);

        self.upload_queue.insert(material_id);

        self.resource_id_to_material_id
            .insert(material_resource_id, material_id);

        self.entity_to_material_id.insert(entity, material_id);

        self.material_id_to_texture_ids.insert(
            material_id,
            Self::collect_texture_dependencies(&material_component.material_data),
        );
    }

    fn change_material(&mut self, entity: Entity, material_component: &MaterialComponent) {
        let material_id = *self.entity_to_material_id.get(&entity).unwrap();

        let mut material = self.get_material_mut(material_id);

        material.material_data = material_component.material_data.clone();

        self.upload_queue.insert(material_id);

        let texture_ids = Self::collect_texture_dependencies(&material_component.material_data);

        self.material_id_to_texture_ids
            .insert(material_id, texture_ids);
    }

    fn remove_material(&mut self, entity: Entity) {
        // TODO(vdbdd): not tested
        let material_id = self.entity_to_material_id.remove(&entity).unwrap();
        self.material_id_to_texture_ids.remove(&material_id);
        self.gpu_material_data.remove_gpu_data(&material_id);
        self.allocator.release_index(material_id.index());
    }

    fn on_texture_state_changed(&mut self, texture_id: &ResourceTypeAndId) {
        // TODO(vdbdd): can be optimized by having a map ( texture_id -> material )
        for (material_id, texture_ids) in &self.material_id_to_texture_ids {
            if texture_ids.contains(texture_id) {
                self.upload_queue.insert(*material_id);
            }
        }
    }

    fn collect_texture_dependencies(material_data: &MaterialData) -> Vec<ResourceTypeAndId> {
        let mut result = Vec::new();

        if material_data.albedo_texture.is_some() {
            result.push(material_data.albedo_texture.as_ref().unwrap().id());
        }
        if material_data.normal_texture.is_some() {
            result.push(material_data.normal_texture.as_ref().unwrap().id());
        }
        if material_data.metalness_texture.is_some() {
            result.push(material_data.metalness_texture.as_ref().unwrap().id());
        }
        if material_data.roughness_texture.is_some() {
            result.push(material_data.roughness_texture.as_ref().unwrap().id());
        }

        result
    }

    fn build_gpu_material_data(
        material_component: &MaterialData,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
        sampler_manager: &SamplerManager,
    ) -> crate::cgen::cgen_type::MaterialData {
        let mut material_data = crate::cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(material_component.base_albedo.r) / 255.0f32,
            f32::from(material_component.base_albedo.g) / 255.0f32,
            f32::from(material_component.base_albedo.b) / 255.0f32,
            f32::from(material_component.base_albedo.a) / 255.0f32,
        );
        material_data.set_base_albedo(color.into());
        material_data.set_base_metalness(material_component.base_metalness.into());
        material_data.set_reflectance(material_component.reflectance.into());
        material_data.set_base_roughness(material_component.base_roughness.into());
        material_data.set_albedo_texture(
            Self::get_bindless_index(
                material_component.albedo_texture.as_ref(),
                SharedTextureId::Albedo,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_normal_texture(
            Self::get_bindless_index(
                material_component.normal_texture.as_ref(),
                SharedTextureId::Normal,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_metalness_texture(
            Self::get_bindless_index(
                material_component.metalness_texture.as_ref(),
                SharedTextureId::Metalness,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_roughness_texture(
            Self::get_bindless_index(
                material_component.roughness_texture.as_ref(),
                SharedTextureId::Roughness,
                texture_manager,
                shared_resources_manager,
            )
            .into(),
        );
        material_data.set_sampler(
            sampler_manager
                .get_index(material_component.sampler.as_ref())
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

    fn upload_material_data(
        &mut self,
        renderer: &Renderer,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
        missing_visuals_tracker: &mut MissingVisualTracker,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
        sampler_manager: &SamplerManager,
    ) {
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

        for material_id in &self.upload_queue {
            let material = &self.get_material(*material_id);
            let material_data = &material.material_data;

            sampler_manager
                .upload_sampler_data(persistent_descriptor_set_manager, material_data.sampler);

            let gpu_material_data = Self::build_gpu_material_data(
                material_data,
                texture_manager,
                shared_resources_manager,
                sampler_manager,
            );
            self.gpu_material_data
                .update_gpu_data(material_id, &gpu_material_data, &mut updater);

            // TODO(vdbdd): remove asap
            missing_visuals_tracker.add_changed_resource(material.resource_id);
        }

        self.upload_queue.clear();

        renderer.add_update_job_block(updater.job_blocks());
    }

    fn upload_default_material(
        &mut self,
        renderer: &Renderer,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        if !self.default_uploaded {
            let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

            let material_data = MaterialData::default();

            let mut default_material_data = crate::cgen::cgen_type::MaterialData::default();
            default_material_data.set_base_albedo(Vec4::new(0.8, 0.8, 0.8, 1.0).into());
            default_material_data.set_base_metalness(0.0.into());
            default_material_data.set_reflectance(0.5.into());
            default_material_data.set_base_roughness(0.4.into());
            default_material_data.set_albedo_texture(
                shared_resources_manager
                    .default_texture_bindless_index(SharedTextureId::Albedo)
                    .into(),
            );
            default_material_data.set_normal_texture(
                shared_resources_manager
                    .default_texture_bindless_index(SharedTextureId::Normal)
                    .into(),
            );
            default_material_data.set_metalness_texture(
                shared_resources_manager
                    .default_texture_bindless_index(SharedTextureId::Metalness)
                    .into(),
            );
            default_material_data.set_roughness_texture(
                shared_resources_manager
                    .default_texture_bindless_index(SharedTextureId::Roughness)
                    .into(),
            );
            default_material_data.set_sampler();

            self.alloc_material(
                self.default_material_id,
                self.default_resource_id,
                material_data, // TODO(vdbdd): default data not in sync with default_material_data
                renderer.static_buffer_allocator(),
            );

            self.gpu_material_data.update_gpu_data(
                &self.default_material_id,
                &default_material_data,
                &mut updater,
            );

            renderer.add_update_job_block(updater.job_blocks());

            self.default_uploaded = true;
        }
    }

    fn alloc_material(
        &mut self,
        material_id: MaterialId,
        resource_id: ResourceTypeAndId,
        material_data: MaterialData,
        gpu_allocator: &UnifiedStaticBufferAllocator,
    ) {
        if self.materials.len() < material_id.index() as usize {
            let next_size =
                round_size_up_to_alignment_u32(material_id.index(), MATERIAL_BLOCK_SIZE);
            self.materials
                .resize(next_size as usize, MaterialSlot::Empty);
        }

        self.gpu_material_data
            .alloc_gpu_data(&material_id, gpu_allocator);

        self.materials[material_id.index() as usize] = MaterialSlot::Occupied(Material {
            va: self.gpu_material_data.va_for_key(&material_id),
            resource_id,
            material_data,
        });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_material_added(
    mut commands: Commands<'_, '_>,
    renderer: Res<'_, Renderer>,
    mut material_manager: ResMut<'_, MaterialManager>,
    query: Query<'_, '_, (Entity, &MaterialComponent), Added<MaterialComponent>>,
) {
    for (entity, material_component) in query.iter() {
        material_manager.add_material(
            entity,
            material_component,
            renderer.static_buffer_allocator(),
        );

        commands
            .entity(entity)
            .insert(GPUMaterialComponent::default());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_material_changed(
    mut material_manager: ResMut<'_, MaterialManager>,
    query: Query<
        '_,
        '_,
        (Entity, &MaterialComponent, &GPUMaterialComponent),
        Changed<MaterialComponent>,
    >,
) {
    for (entity, material_component, _) in query.iter() {
        material_manager.change_material(entity, material_component);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_material_removed(
    removed_entities: RemovedComponents<'_, MaterialComponent>,
    mut material_manager: ResMut<'_, MaterialManager>,
) {
    // todo: must be send some events to refresh the material
    for removed_entity in removed_entities.iter() {
        material_manager.remove_material(removed_entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_texture_event(
    mut event_reader: EventReader<'_, '_, TextureEvent>,
    mut material_manager: ResMut<'_, MaterialManager>,
) {
    for event in event_reader.iter() {
        match event {
            TextureEvent::StateChanged(texture_id_list) => {
                for texture_id in texture_id_list {
                    material_manager.on_texture_state_changed(texture_id);
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn upload_default_material(
    mut material_manager: ResMut<'_, MaterialManager>,
    renderer: Res<'_, Renderer>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
) {
    material_manager.upload_default_material(&renderer, &shared_resources_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn upload_material_data(
    renderer: Res<'_, Renderer>,
    mut material_manager: ResMut<'_, MaterialManager>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    material_manager.upload_material_data(
        &renderer,
        &texture_manager,
        &shared_resources_manager,
        &mut missing_visuals_tracker,
    );
}
