use std::collections::BTreeMap;

use lgn_app::App;
use lgn_data_runtime::{from_binary_reader, prelude::*};
use lgn_ecs::prelude::*;
use lgn_graphics_api::{AddressMode, CompareOp, FilterType, MipMapMode, SamplerDef};
use lgn_graphics_data::{runtime::BinTextureReferenceType, runtime::SamplerData};
use lgn_math::Vec4;
use lgn_utils::{memory::round_size_up_to_alignment_u32, HashSet};

use crate::{
    components::{MaterialComponent, MaterialData},
    labels::RenderStage,
    resources::SharedTextureId,
    Renderer, ResourceStageLabel,
};

use super::{
    GpuDataManager, IndexAllocator, MissingVisualTracker, SamplerId, SamplerManager,
    SharedResourcesManager, TextureEvent, TextureManager, UnifiedStaticBufferAllocator,
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

#[allow(clippy::large_enum_variant)]
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

pub(crate) struct MaterialInstaller {}
impl MaterialInstaller {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ResourceInstaller for MaterialInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let material = from_binary_reader::<lgn_graphics_data::runtime::Material>(reader).await?;
        lgn_tracing::info!("Material {}", resource_id.id,);

        let handle = request.asset_registry.set_resource(resource_id, material)?;
        Ok(handle)

        /*let mut entity = if let Some(entity) = asset_to_entity_map.get(asset_handle.id()) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        let asset_id = asset_handle.id();
        let material = asset_handle.get()?;
        let albedo = material.albedo.clone();
        let normal = material.normal.clone();
        let metalness = material.metalness.clone();
        let roughness = material.roughness.clone();
        std::mem::forget(material);
        entity.insert(MaterialComponent::new(
            asset_handle,
            albedo,
            normal,
            metalness,
            roughness,
        ));

        info!(
            "Spawned {}: {} -> ECS id: {:?}",
            asset_id.kind.as_pretty().trim_start_matches("runtime_"),
            asset_id.id,
            entity.id(),
        );
        Some(entity.id())*/
    }
}

const MATERIAL_BLOCK_SIZE: u32 = 2048;

pub struct MaterialManager {
    index_allocator: IndexAllocator,
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
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        let mut index_allocator = IndexAllocator::new(MATERIAL_BLOCK_SIZE);

        // TODO(vdbdd): redundant and useless. remove asap.
        let default_resource_id = ResourceTypeAndId {
            kind: lgn_graphics_data::runtime::Material::TYPE,
            id: ResourceId::new(),
        };

        let default_material_id = index_allocator.acquire_index();

        Self {
            index_allocator,
            materials: Vec::new(),
            resource_id_to_material_id: BTreeMap::new(),
            entity_to_material_id: BTreeMap::new(),
            material_id_to_texture_ids: BTreeMap::new(),
            upload_queue: HashSet::new(),
            gpu_material_data: GpuMaterialData::new(allocator, MATERIAL_BLOCK_SIZE),
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

    fn add_material(&mut self, entity: Entity, material_component: &MaterialComponent) {
        let material_resource_id = material_component.resource.id();
        let material_id: MaterialId = self.index_allocator.acquire_index().into();
        let material_data = material_component.material_data.clone();

        self.alloc_material(material_id, material_resource_id, material_data);

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
        self.index_allocator.release_index(material_id.index());
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
            Self::get_sampler_index(sampler_manager, material_component.sampler_data.as_ref())
                .as_index()
                .into(),
        );

        material_data
    }

    fn get_bindless_index(
        texture_id: Option<&BinTextureReferenceType>,
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

    fn get_sampler_index(
        sampler_manager: &SamplerManager,
        sampler_data: Option<&SamplerData>,
    ) -> SamplerId {
        if let Some(sampler_data) = sampler_data {
            #[allow(clippy::match_same_arms)]
            sampler_manager.get_index(&SamplerDef {
                min_filter: match sampler_data.min_filter {
                    lgn_graphics_data::Filter::Nearest => FilterType::Nearest,
                    lgn_graphics_data::Filter::Linear => FilterType::Linear,
                    _ => FilterType::Linear,
                },
                mag_filter: match sampler_data.mag_filter {
                    lgn_graphics_data::Filter::Nearest => FilterType::Nearest,
                    lgn_graphics_data::Filter::Linear => FilterType::Linear,
                    _ => FilterType::Linear,
                },
                mip_map_mode: match sampler_data.mip_filter {
                    lgn_graphics_data::Filter::Nearest => MipMapMode::Nearest,
                    lgn_graphics_data::Filter::Linear => MipMapMode::Linear,
                    _ => MipMapMode::Linear,
                },
                address_mode_u: match sampler_data.wrap_u {
                    lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                    lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                    lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                    _ => AddressMode::Repeat,
                },
                address_mode_v: match sampler_data.wrap_v {
                    lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                    lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                    lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                    _ => AddressMode::Repeat,
                },
                address_mode_w: AddressMode::Repeat,
                mip_lod_bias: 0.0,
                max_anisotropy: 1.0,
                compare_op: CompareOp::LessOrEqual,
            })
        } else {
            SamplerManager::get_default_sampler_index()
        }
    }

    fn upload_material_data(
        &mut self,
        renderer: &Renderer,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
        missing_visuals_tracker: &mut MissingVisualTracker,
        sampler_manager: &SamplerManager,
    ) {
        let mut render_commands = renderer.render_command_builder();

        for material_id in &self.upload_queue {
            let material = &self.get_material(*material_id);
            let material_data = &material.material_data;

            let gpu_material_data = Self::build_gpu_material_data(
                material_data,
                texture_manager,
                shared_resources_manager,
                sampler_manager,
            );
            self.gpu_material_data.update_gpu_data(
                material_id,
                &gpu_material_data,
                &mut render_commands,
            );

            // TODO(vdbdd): remove asap
            missing_visuals_tracker.add_changed_resource(material.resource_id);
        }

        self.upload_queue.clear();
    }

    fn upload_default_material(
        &mut self,
        renderer: &Renderer,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        if !self.default_uploaded {
            let mut render_commands = renderer.render_command_builder();

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
            default_material_data.set_sampler(
                SamplerManager::get_default_sampler_index()
                    .as_index()
                    .into(),
            );

            self.alloc_material(
                self.default_material_id,
                self.default_resource_id,
                material_data, // TODO(vdbdd): default data not in sync with default_material_data
            );

            self.gpu_material_data.update_gpu_data(
                &self.default_material_id,
                &default_material_data,
                &mut render_commands,
            );

            self.default_uploaded = true;
        }
    }

    fn alloc_material(
        &mut self,
        material_id: MaterialId,
        resource_id: ResourceTypeAndId,
        material_data: MaterialData,
    ) {
        if material_id.index() as usize >= self.materials.len() {
            let next_size =
                round_size_up_to_alignment_u32(material_id.index() + 1, MATERIAL_BLOCK_SIZE);
            self.materials
                .resize(next_size as usize, MaterialSlot::Empty);
        }

        self.gpu_material_data.alloc_gpu_data(&material_id);

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
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
    query: Query<'_, '_, (Entity, &MaterialComponent), Added<MaterialComponent>>,
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
    for (entity, material_component) in query.iter() {
        material_manager.add_material(entity, material_component);

        commands
            .entity(entity)
            .insert(GPUMaterialComponent::default());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_material_changed(
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
    query: Query<
        '_,
        '_,
        (Entity, &MaterialComponent, &GPUMaterialComponent),
        Changed<MaterialComponent>,
    >,
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
    for (entity, material_component, _) in query.iter() {
        material_manager.change_material(entity, material_component);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_material_removed(
    removed_entities: RemovedComponents<'_, MaterialComponent>,
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
    // todo: must be send some events to refresh the material
    for removed_entity in removed_entities.iter() {
        material_manager.remove_material(removed_entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_texture_event(
    mut event_reader: EventReader<'_, '_, TextureEvent>,
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
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
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
    shared_resources_manager: Res<'_, SharedResourcesManager>,
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
    material_manager.upload_default_material(&renderer, &shared_resources_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn upload_material_data(
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
) {
    let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
    let mut missing_visuals_tracker = renderer
        .render_resources()
        .get_mut::<MissingVisualTracker>();

    let sampler_manager = renderer.render_resources().get::<SamplerManager>();
    material_manager.upload_material_data(
        &renderer,
        &texture_manager,
        &shared_resources_manager,
        &mut missing_visuals_tracker,
        &sampler_manager,
    );
}
