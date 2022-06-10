use std::collections::BTreeMap;

use async_trait::async_trait;
use lgn_app::App;
use lgn_data_runtime::{
    from_binary_reader, AssetRegistryError, AssetRegistryReader, ComponentInstaller, LoadRequest,
    ResourceInstaller, ResourceTypeAndId,
};
use lgn_ecs::{prelude::*, schedule::SystemLabel, system::EntityCommands};
use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, TextureDef,
    TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::TextureFormat;
use lgn_tracing::span_fn;

use crate::{
    components::{LightComponent, TextureComponent, TextureData, VisualComponent},
    core::UploadTextureCommand,
    labels::RenderStage,
    Renderer, ResourceStageLabel,
};

use super::PersistentDescriptorSetManager;

pub enum TextureEvent {
    StateChanged(Vec<ResourceTypeAndId>),
}

struct UploadTextureJob {
    entity: Entity,
    texture_data: TextureData,
}

struct RemoveTextureJob {
    entity: Entity,
    texture_id: ResourceTypeAndId,
}

enum TextureJob {
    Upload(UploadTextureJob),
    Remove(RemoveTextureJob),
}

#[derive(Component)]
struct GPUTextureComponent;

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
    texture_infos: BTreeMap<Entity, TextureInfo>,
    // todo: use some kind of queue maybe?
    texture_jobs: Vec<TextureJob>,
    texture_id_to_entity: BTreeMap<ResourceTypeAndId, Entity>,
}

pub(crate) struct TextureInstaller {
    //texture_manager: Arc<TextureManager>,
}

impl TextureInstaller {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ComponentInstaller for TextureInstaller {
    /// Consume a resource return the installed version
    fn install_component(
        &self,
        component: &dyn lgn_data_runtime::Component,
        entity_command: &mut EntityCommands<'_, '_, '_>,
    ) -> Result<(), AssetRegistryError> {
        // Visual Test
        if let Some(visual) = component.downcast_ref::<lgn_graphics_data::runtime::Visual>() {
            entity_command.insert(VisualComponent::new(
                visual.renderable_geometry.as_ref().map(|r| r.id()),
                visual.color,
                visual.color_blend,
            ));
            entity_command.insert(visual.clone()); // Add to keep Model alive
        } else if let Some(light) = component.downcast_ref::<lgn_graphics_data::runtime::Light>() {
            entity_command.insert(LightComponent {
                light_type: match light.light_type {
                    lgn_graphics_data::LightType::Omnidirectional => {
                        crate::components::LightType::OmniDirectional
                    }
                    lgn_graphics_data::LightType::Directional => {
                        crate::components::LightType::Directional
                    }
                    lgn_graphics_data::LightType::Spotlight => crate::components::LightType::Spot,
                    _ => unreachable!("Unrecognized light type"),
                },
                color: light.color,
                radiance: light.radiance,
                cone_angle: light.cone_angle,
                enabled: light.enabled,
                ..LightComponent::default()
            });
        } else if let Some(camera_setup) =
            component.downcast_ref::<lgn_graphics_data::runtime::CameraSetup>()
        {
            entity_command.insert(camera_setup.clone());
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceInstaller for TextureInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<lgn_data_runtime::HandleUntyped, AssetRegistryError> {
        let texture = from_binary_reader::<lgn_graphics_data::runtime::BinTexture>(reader).await?;
        lgn_tracing::info!(
            "Texture {} | width: {}, height: {}, format: {:?}",
            resource_id.id,
            texture.width,
            texture.height,
            texture.format
        );
        let handle = request.asset_registry.set_resource(resource_id, texture)?;

        Ok(handle)

        /*let mut entity = if let Some(entity) = asset_to_entity_map.get(resource.id()) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        let texture = resource.get()?;
        let texture_mips = texture // TODO: Avoid cloning in the future
            .texture_data
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();

        let texture_data = TextureData::from_slices(&texture_mips);
        let asset_id = resource.id();
        let width = texture.width;
        let height = texture.height;
        let format = texture.format;
        let srgb = texture.srgb;
        std::mem::forget(texture);
        let texture_component =
            TextureComponent::new(resource, width, height, format, srgb, texture_data);

        entity.insert(texture_component);
        info!(
            "Spawned {}: {} -> ECS id: {:?} | width: {}, height: {}, format: {:?}",
            asset_id.kind.as_pretty().trim_start_matches("runtime_"),
            asset_id.id,
            entity.id(),
            width,
            height,
            format
        );
        Some(entity.id())*/
    }
}

impl TextureManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            texture_infos: BTreeMap::new(),
            texture_jobs: Vec::new(),
            texture_id_to_entity: BTreeMap::new(),
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_event::<TextureEvent>();

        app.add_system_set_to_stage(
            RenderStage::Resource,
            SystemSet::new()
                .with_system(on_texture_added)
                .with_system(on_texture_modified)
                .with_system(on_texture_removed)
                .label(TextureManagerLabel::UpdateDone),
        );

        app.add_system_to_stage(
            RenderStage::Resource,
            apply_changes
                .label(ResourceStageLabel::Texture)
                .after(TextureManagerLabel::UpdateDone),
        );
    }

    pub fn allocate_texture(&mut self, entity: Entity, texture_component: &TextureComponent) {
        let texture_def = Self::texture_def_from_texture_component(texture_component);

        let texture_view = self.create_texture_view(&texture_def, "material_texture");

        self.texture_infos.insert(
            entity,
            TextureInfo {
                state: TextureState::Invalid,
                texture_id: texture_component.resource.id(),
                bindless_index: None,
                texture_view,
            },
        );

        self.texture_id_to_entity
            .insert(texture_component.resource.id(), entity);

        self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
            entity,
            texture_data: texture_component.texture_data.clone(),
        }));

        let texture_info = self.texture_info_mut(entity);
        texture_info.state = TextureState::QueuedForUpload;
    }

    pub fn update_texture(&mut self, entity: Entity, texture_component: &TextureComponent) {
        // TODO(vdbdd): not tested
        assert_eq!(self.texture_info(entity).state, TextureState::Ready);

        let texture_def = Self::texture_def_from_texture_component(texture_component);

        let recreate_texture_view = {
            let texture_info = self.texture_info(entity);
            let current_texture_handle = texture_info.texture_view.texture();
            let current_texture_def = current_texture_handle.definition();
            *current_texture_def != texture_def
        };

        if recreate_texture_view {
            let texture_view = self.create_texture_view(&texture_def, "material_texture");
            let texture_info = self.texture_info_mut(entity);
            texture_info.texture_view = texture_view;
        }

        self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
            entity,
            texture_data: texture_component.texture_data.clone(),
        }));

        let texture_info = self.texture_info_mut(entity);
        texture_info.state = TextureState::QueuedForUpload;
    }

    pub fn remove_by_entity(&mut self, entity: Entity) {
        // TODO(vdbdd): not tested
        assert_eq!(self.texture_info(entity).state, TextureState::Ready);

        let texture_id = self.texture_info(entity).texture_id;
        self.texture_jobs
            .push(TextureJob::Remove(RemoveTextureJob { entity, texture_id }));

        self.texture_infos.remove(&entity);
    }

    pub fn bindless_index_for_resource_id(&self, texture_id: &ResourceTypeAndId) -> Option<u32> {
        let entity = self.texture_id_to_entity.get(texture_id);
        if let Some(entity) = entity {
            let texture_info = self.texture_infos.get(entity);
            texture_info.map(|ti| ti.bindless_index).and_then(|ti| ti)
        } else {
            None
        }
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
                    let bindless_index = self.texture_info(upload_job.entity).bindless_index;

                    if let Some(bindless_index) = bindless_index {
                        persistent_descriptor_set_manager.unset_bindless_texture(bindless_index);
                    }

                    self.upload_texture(renderer, upload_job);

                    let texture_info = self.texture_info_mut(upload_job.entity);
                    texture_info.state = TextureState::Ready;
                    texture_info.bindless_index = Some(
                        persistent_descriptor_set_manager
                            .set_bindless_texture(&texture_info.texture_view),
                    );

                    state_changed_list.push(texture_info.texture_id);
                }
                TextureJob::Remove(remove_job) => {
                    // TODO(vdbdd): not tested
                    let texture_info = self.texture_infos.get_mut(&remove_job.entity).unwrap();

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

    fn is_valid(&self, entity: Entity) -> bool {
        self.texture_infos.contains_key(&entity)
    }

    #[span_fn]
    fn upload_texture(&mut self, renderer: &Renderer, upload_job: &UploadTextureJob) {
        let texture_info = self.texture_infos.get(&upload_job.entity).unwrap();

        let mut render_commands = renderer.render_command_builder();

        render_commands.push(UploadTextureCommand {
            src_data: upload_job.texture_data.clone(),
            dst_texture: texture_info.texture_view.texture().clone(),
        });
    }

    fn create_texture_view(&self, texture_def: &TextureDef, name: &str) -> TextureView {
        // todo: bundle all the default views in the resource instead of having them separatly
        let texture = self.device_context.create_texture(*texture_def, name);
        texture.create_view(TextureViewDef::as_shader_resource_view(texture_def))
    }

    fn texture_info(&self, entity: Entity) -> &TextureInfo {
        assert!(self.is_valid(entity));
        self.texture_infos.get(&entity).unwrap()
    }

    fn texture_info_mut(&mut self, entity: Entity) -> &mut TextureInfo {
        assert!(self.is_valid(entity));
        self.texture_infos.get_mut(&entity).unwrap()
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
            _ => {
                panic!("Unsupported format");
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
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
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
        texture_manager.update_texture(entity, texture_component);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_texture_removed(
    mut commands: Commands<'_, '_>,
    removed_entities: RemovedComponents<'_, TextureComponent>,
    mut texture_manager: ResMut<'_, TextureManager>,
) {
    // todo: must be send some events to refresh the material
    for removed_entity in removed_entities.iter() {
        commands
            .entity(removed_entity)
            .remove::<GPUTextureComponent>();
        texture_manager.remove_by_entity(removed_entity);
    }
}

#[allow(clippy::needless_pass_by_value)]
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
