use std::sync::Arc;

use async_trait::async_trait;

use lgn_data_runtime::{
    from_binary_reader, AssetRegistryError, AssetRegistryReader, ComponentInstaller, LoadRequest,
    Resource, ResourceInstaller, ResourceTypeAndId,
};
use lgn_ecs::system::EntityCommands;
use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, Texture,
    TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::{runtime::BinTexture, TextureFormat};

use crate::components::{LightComponent, VisualComponent};

use super::{PersistentDescriptorSetManager, TextureSlot};

#[derive(Clone)]
pub struct TextureData {
    data: Arc<Vec<Vec<u8>>>,
}

impl TextureData {
    pub fn from_slice<T: Sized>(mip0_data: &[T]) -> Self {
        Self {
            data: Arc::new(vec![Self::to_vec_u8(mip0_data)]),
        }
    }

    pub fn from_slices<T: Sized>(mips_data: &[&[T]]) -> Self {
        let mut data = Vec::with_capacity(mips_data.len());
        for mip_data in mips_data.iter() {
            data.push(Self::to_vec_u8(*mip_data));
        }

        Self {
            data: Arc::new(data),
        }
    }

    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    pub fn mip_count(&self) -> usize {
        self.data.len()
    }

    #[allow(unsafe_code)]
    fn to_vec_u8<T: Sized>(mip_data: &[T]) -> Vec<u8> {
        let src_ptr = mip_data.as_ptr().cast::<u8>();
        let src_size = mip_data.len() * std::mem::size_of::<T>();
        unsafe {
            let dst_ptr =
                std::alloc::alloc(std::alloc::Layout::from_size_align(src_size, 16).unwrap());
            dst_ptr.copy_from_nonoverlapping(src_ptr, src_size);
            Vec::<u8>::from_raw_parts(dst_ptr, src_size, src_size)
        }
    }
}
#[derive(Clone)]
struct TextureInfo {
    bindless_index: Option<u32>,
    texture_view: TextureView,
}

struct Inner {
    device_context: DeviceContext,
    persistent_descriptor_set_manager: PersistentDescriptorSetManager,
}

#[derive(Clone)]
pub struct TextureManager {
    inner: Arc<Inner>,
}

pub struct TextureInstaller {
    texture_manager: TextureManager,
}

impl TextureInstaller {
    pub(crate) fn new(texture_manager: &TextureManager) -> Self {
        Self {
            texture_manager: texture_manager.clone(),
        }
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

#[derive(Clone)]
pub struct RenderTexture {
    data: Box<lgn_graphics_data::runtime::BinTexture>,
    gpu_texture: Texture,
    default_gpu_view: TextureView,
    bindless_slot: TextureSlot,
}
lgn_data_runtime::implement_runtime_resource!(RenderTexture);

impl RenderTexture {
    pub fn data(&self) -> &BinTexture {
        &self.data
    }

    pub fn gpu_texture(&self) -> &Texture {
        &self.gpu_texture
    }

    pub fn bindless_slot(&self) -> TextureSlot {
        self.bindless_slot
    }
}

#[async_trait]
impl ResourceInstaller for TextureInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        _request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<Box<dyn Resource>, AssetRegistryError> {
        let data = from_binary_reader::<lgn_graphics_data::runtime::BinTexture>(reader).await?;
        lgn_tracing::info!(
            "Texture {} | width: {}, height: {}, format: {:?}",
            resource_id.id,
            data.width,
            data.height,
            data.format
        );

        let render_texture = Box::new(
            self.texture_manager
                .create_texture(data, &resource_id.to_string()),
        );

        Ok(render_texture)
    }
}

impl TextureManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                device_context: device_context.clone(),
                persistent_descriptor_set_manager: persistent_descriptor_set_manager.clone(),
            }),
        }
    }

    pub fn create_texture(&self, data: Box<BinTexture>, name: &str) -> RenderTexture {
        let texture_def = Self::texture_def_from_data(&data);
        let gpu_texture = self.inner.device_context.create_texture(texture_def, name);
        let default_gpu_view =
            gpu_texture.create_view(TextureViewDef::as_shader_resource_view(&texture_def));
        let bindless_index = self
            .inner
            .persistent_descriptor_set_manager
            .allocate_texture_slot(&default_gpu_view);

        RenderTexture {
            data,
            gpu_texture,
            default_gpu_view,
            bindless_slot: bindless_index,
        }
    }

    // pub fn update_texture(&mut self, entity: Entity, texture_component: &TextureComponent) {
    //     // TODO(vdbdd): not tested
    //     assert_eq!(self.texture_info(entity).state, TextureState::Ready);

    //     let texture_def = Self::texture_def_from_texture_component(texture_component);

    //     let recreate_texture_view = {
    //         let texture_info = self.texture_info(entity);
    //         let current_texture_handle = texture_info.texture_view.texture();
    //         let current_texture_def = current_texture_handle.definition();
    //         *current_texture_def != texture_def
    //     };

    //     if recreate_texture_view {
    //         let texture_view = self.create_texture_view(&texture_def, "material_texture");
    //         let texture_info = self.texture_info_mut(entity);
    //         texture_info.texture_view = texture_view;
    //     }

    //     self.texture_jobs.push(TextureJob::Upload(UploadTextureJob {
    //         entity,
    //         texture_data: texture_component.texture_data.clone(),
    //     }));

    //     let texture_info = self.texture_info_mut(entity);
    //     texture_info.state = TextureState::QueuedForUpload;
    // }

    // pub fn remove_by_entity(&mut self, entity: Entity) {
    //     // TODO(vdbdd): not tested
    //     assert_eq!(self.texture_info(entity).state, TextureState::Ready);

    //     let texture_id = self.texture_info(entity).texture_id;
    //     self.texture_jobs
    //         .push(TextureJob::Remove(RemoveTextureJob { entity, texture_id }));

    //     self.texture_infos.remove(&entity);
    // }

    // pub fn bindless_index_for_resource_id(&self, texture_id: &ResourceTypeAndId) -> Option<u32> {
    //     let entity = self.texture_id_to_entity.get(texture_id);
    //     if let Some(entity) = entity {
    //         let texture_info = self.texture_infos.get(entity);
    //         texture_info.map(|ti| ti.bindless_index).and_then(|ti| ti)
    //     } else {
    //         None
    //     }
    // }

    // #[span_fn]
    // pub fn apply_changes(
    //     &mut self,
    //     renderer: &Renderer,
    //     persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    // ) -> Vec<ResourceTypeAndId> {
    //     if self.texture_jobs.is_empty() {
    //         return Vec::new();
    //     }

    //     // TODO(vdbdd): remove this heap allocation
    //     let mut state_changed_list = Vec::with_capacity(self.texture_jobs.len());
    //     let mut texture_jobs = std::mem::take(&mut self.texture_jobs);

    //     for texture_job in &texture_jobs {
    //         match texture_job {
    //             TextureJob::Upload(upload_job) => {
    //                 let bindless_index = self.texture_info(upload_job.entity).bindless_index;

    //                 if let Some(bindless_index) = bindless_index {
    //                     persistent_descriptor_set_manager.unset_bindless_texture(bindless_index);
    //                 }

    //                 self.upload_texture(renderer, upload_job);

    //                 let texture_info = self.texture_info_mut(upload_job.entity);
    //                 texture_info.state = TextureState::Ready;
    //                 texture_info.bindless_index = Some(
    //                     persistent_descriptor_set_manager
    //                         .set_bindless_texture(&texture_info.texture_view),
    //                 );

    //                 state_changed_list.push(texture_info.texture_id);
    //             }
    //             TextureJob::Remove(remove_job) => {
    //                 // TODO(vdbdd): not tested
    //                 let texture_info = self.texture_infos.get_mut(&remove_job.entity).unwrap();

    //                 let bindless_index = texture_info.bindless_index.unwrap();
    //                 persistent_descriptor_set_manager.unset_bindless_texture(bindless_index);

    //                 state_changed_list.push(remove_job.texture_id);
    //             }
    //         }
    //     }

    //     texture_jobs.clear();

    //     self.texture_jobs = texture_jobs;

    //     state_changed_list
    // }

    // // fn is_valid(&self, entity: Entity) -> bool {
    // //     self.texture_infos.contains_key(&entity)
    // // }

    // #[span_fn]
    // fn upload_texture(&mut self, renderer: &Renderer, upload_job: &UploadTextureJob) {
    //     let texture_info = self.texture_infos.get(&upload_job.entity).unwrap();

    //     let mut render_commands = renderer.render_command_builder();

    //     render_commands.push(UploadTextureCommand {
    //         src_data: upload_job.texture_data.clone(),
    //         dst_texture: texture_info.texture_view.texture().clone(),
    //     });
    // }

    // fn texture_info(&self, entity: Entity) -> &TextureInfo {
    //     assert!(self.is_valid(entity));
    //     self.texture_infos.get(&entity).unwrap()
    // }

    // fn texture_info_mut(&mut self, entity: Entity) -> &mut TextureInfo {
    //     assert!(self.is_valid(entity));
    //     self.texture_infos.get_mut(&entity).unwrap()
    // }

    fn texture_def_from_data(texture_data: &BinTexture) -> TextureDef {
        let format = match texture_data.format {
            TextureFormat::BC1 => {
                if texture_data.srgb {
                    Format::BC1_RGBA_SRGB_BLOCK
                } else {
                    Format::BC1_RGBA_UNORM_BLOCK
                }
            }
            TextureFormat::BC3 => {
                if texture_data.srgb {
                    Format::BC3_SRGB_BLOCK
                } else {
                    Format::BC3_UNORM_BLOCK
                }
            }
            TextureFormat::BC4 => {
                assert!(!texture_data.srgb);
                Format::BC4_UNORM_BLOCK
            }
            TextureFormat::BC7 => {
                if texture_data.srgb {
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
                width: texture_data.width,
                height: texture_data.height,
                depth: 1,
            },
            array_length: 1,
            mip_count: texture_data.mips.len() as u32,
            format,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

// #[allow(clippy::needless_pass_by_value)]
// fn on_texture_added(
//     mut commands: Commands<'_, '_>,
//     mut texture_manager: ResMut<'_, TextureManager>,
//     q_added_textures: Query<'_, '_, (Entity, &TextureComponent), Added<TextureComponent>>,
// ) {
//     if q_added_textures.is_empty() {
//         return;
//     }

//     for (entity, texture_component) in q_added_textures.iter() {
//         texture_manager.allocate_texture(entity, texture_component);

//         commands.entity(entity).insert(GPUTextureComponent);
//     }
// }

// #[allow(clippy::needless_pass_by_value)]
// fn on_texture_modified(
//     mut texture_manager: ResMut<'_, TextureManager>,
//     q_modified_textures: Query<
//         '_,
//         '_,
//         (Entity, &TextureComponent, &GPUTextureComponent),
//         Changed<TextureComponent>,
//     >,
// ) {
//     if q_modified_textures.is_empty() {
//         return;
//     }

//     for (entity, texture_component, _) in q_modified_textures.iter() {
//         texture_manager.update_texture(entity, texture_component);
//     }
// }

// #[allow(clippy::needless_pass_by_value)]
// fn on_texture_removed(
//     mut commands: Commands<'_, '_>,
//     removed_entities: RemovedComponents<'_, TextureComponent>,
//     mut texture_manager: ResMut<'_, TextureManager>,
// ) {
//     // todo: must be send some events to refresh the material
//     for removed_entity in removed_entities.iter() {
//         commands
//             .entity(removed_entity)
//             .remove::<GPUTextureComponent>();
//         texture_manager.remove_by_entity(removed_entity);
//     }
// }

// #[allow(clippy::needless_pass_by_value)]
// fn apply_changes(
//     mut event_writer: EventWriter<'_, '_, TextureEvent>,
//     renderer: Res<'_, Renderer>,
//     mut texture_manager: ResMut<'_, TextureManager>,
//     mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
// ) {
//     // todo: must be send some events to refresh the material
//     let state_changed_list =
//         texture_manager.apply_changes(&renderer, &mut persistent_descriptor_set_manager);
//     if !state_changed_list.is_empty() {
//         event_writer.send(TextureEvent::StateChanged(state_changed_list));
//     }
// }
