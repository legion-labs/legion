use std::collections::BTreeMap;

use lgn_app::App;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::*;

use lgn_graphics_data::runtime_texture::TextureReferenceType;
use lgn_math::Vec4;
use lgn_tracing::span_fn;

use crate::{
    cgen, components::MaterialComponent, labels::RenderStage, resources::SharedTextureId, Renderer,
};

use super::{
    GpuDataManager, SharedResourcesManager, TextureEvent, TextureManager,
    UnifiedStaticBufferAllocator, UniformGPUDataUpdater,
};

#[derive(Default, Component)]
struct GPUMaterialComponent;

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum MaterialLabel {
    UpdateDone,
}

pub(crate) type GpuMaterialData = GpuDataManager<ResourceTypeAndId, cgen::cgen_type::MaterialData>;

struct UploadMaterialJob {
    resource_id: ResourceTypeAndId,
    material_data: cgen::cgen_type::MaterialData,
}

pub struct MaterialManager {
    entity_to_resource_id: BTreeMap<Entity, ResourceTypeAndId>,
    entity_to_texture_ids: BTreeMap<Entity, Vec<ResourceTypeAndId>>,
    upload_queue: Vec<UploadMaterialJob>,
    gpu_material_data: GpuMaterialData,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            entity_to_resource_id: BTreeMap::new(),
            entity_to_texture_ids: BTreeMap::new(),
            upload_queue: Vec::new(),
            gpu_material_data: GpuMaterialData::new(64 * 1024, 256),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(on_material_added)
                .with_system(on_material_changed)
                .with_system(on_material_removed)
                .with_system(on_texture_event)
                .with_system(upload_default_material)
                .label(MaterialLabel::UpdateDone),
        );
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(upload_material_data)
                .after(MaterialLabel::UpdateDone),
        );
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
        entity: Entity,
        material_component: &MaterialComponent,
        allocator: &UnifiedStaticBufferAllocator,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        self.gpu_material_data
            .alloc_gpu_data(material_component.material_id, allocator);

        let job = UploadMaterialJob {
            resource_id: material_component.material_id,
            material_data: Self::material_component_to_material_data(
                material_component,
                texture_manager,
                shared_resources_manager,
            ),
        };
        self.upload_queue.push(job);

        self.entity_to_resource_id
            .insert(entity, material_component.material_id);

        self.entity_to_texture_ids.insert(
            entity,
            Self::collect_texture_dependencies(material_component),
        );
    }

    pub fn change_material(
        &mut self,
        entity: Entity,
        material_component: &MaterialComponent,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        // TODO(vdbdd): not tested
        let job = UploadMaterialJob {
            resource_id: material_component.material_id,
            material_data: Self::material_component_to_material_data(
                material_component,
                texture_manager,
                shared_resources_manager,
            ),
        };
        self.upload_queue.push(job);

        let texture_ids = Self::collect_texture_dependencies(material_component);
        self.entity_to_texture_ids.insert(entity, texture_ids);
    }

    pub fn remove_material(&mut self, entity: Entity) {
        // TODO(vdbdd): not tested
        self.entity_to_texture_ids.remove(&entity);
        let resource_id = self.entity_to_resource_id.remove(&entity).unwrap();
        self.gpu_material_data.remove_gpu_data(&resource_id);
    }

    pub fn on_texture_state_changed(
        &mut self,
        texture_id: &ResourceTypeAndId,
        query_material_components: &Query<'_, '_, &MaterialComponent>,
        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) {
        // todo: can be optimized by having a map ( texture_id -> material )
        for (entity, texture_ids) in &self.entity_to_texture_ids {
            if texture_ids.contains(texture_id) {
                let material_component = query_material_components
                    .get_component::<MaterialComponent>(*entity)
                    .unwrap();
                let job = UploadMaterialJob {
                    resource_id: material_component.material_id,
                    material_data: Self::material_component_to_material_data(
                        material_component,
                        texture_manager,
                        shared_resources_manager,
                    ),
                };
                self.upload_queue.push(job);
            }
        }
    }

    fn collect_texture_dependencies(
        material_component: &MaterialComponent,
    ) -> Vec<ResourceTypeAndId> {
        let mut result = Vec::new();

        if material_component.albedo_texture.is_some() {
            result.push(material_component.albedo_texture.as_ref().unwrap().id());
        }
        if material_component.normal_texture.is_some() {
            result.push(material_component.normal_texture.as_ref().unwrap().id());
        }
        if material_component.metalness_texture.is_some() {
            result.push(material_component.metalness_texture.as_ref().unwrap().id());
        }
        if material_component.roughness_texture.is_some() {
            result.push(material_component.roughness_texture.as_ref().unwrap().id());
        }

        result
    }

    fn material_component_to_material_data(
        material_component: &MaterialComponent,

        texture_manager: &TextureManager,
        shared_resources_manager: &SharedResourcesManager,
    ) -> cgen::cgen_type::MaterialData {
        let mut material_data = cgen::cgen_type::MaterialData::default();

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

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_material_added(
    mut commands: Commands<'_, '_>,
    renderer: Res<'_, Renderer>,
    mut material_manager: ResMut<'_, MaterialManager>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
    query: Query<'_, '_, (Entity, &MaterialComponent), Added<MaterialComponent>>,
) {
    for (entity, material_component) in query.iter() {
        material_manager.add_material(
            entity,
            material_component,
            renderer.static_buffer_allocator(),
            &texture_manager,
            &shared_resources_manager,
        );

        commands
            .entity(entity)
            .insert(GPUMaterialComponent::default());
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_material_changed(
    mut material_manager: ResMut<'_, MaterialManager>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
    query: Query<
        '_,
        '_,
        (Entity, &MaterialComponent, &GPUMaterialComponent),
        Changed<MaterialComponent>,
    >,
) {
    for (entity, material_component, _) in query.iter() {
        material_manager.change_material(
            entity,
            material_component,
            &texture_manager,
            &shared_resources_manager,
        );
    }
}

#[span_fn]
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

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_texture_event(
    mut event_reader: EventReader<'_, '_, TextureEvent>,
    mut material_manager: ResMut<'_, MaterialManager>,
    texture_manager: Res<'_, TextureManager>,
    shared_resources_manager: Res<'_, SharedResourcesManager>,
    query: Query<'_, '_, &MaterialComponent>,
) {
    for event in event_reader.iter() {
        match event {
            TextureEvent::StateChanged(texture_id_list) => {
                for texture_id in texture_id_list {
                    material_manager.on_texture_state_changed(
                        texture_id,
                        &query,
                        &texture_manager,
                        &shared_resources_manager,
                    );
                }
            }
        }
    }
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
