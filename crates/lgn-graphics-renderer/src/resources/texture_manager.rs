use std::sync::Arc;

use async_trait::async_trait;

use crossbeam::atomic::AtomicCell;
use lgn_graphics_data::Color;
use strum::EnumCount;
use strum::IntoEnumIterator;
use uuid::uuid;

use lgn_data_runtime::{
    from_binary_reader, AssetRegistry, AssetRegistryError, AssetRegistryReader, ComponentInstaller,
    Handle, LoadRequest, Resource, ResourceDescriptor, ResourceId, ResourceInstaller,
    ResourceTypeAndId,
};
use lgn_ecs::system::EntityCommands;
use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, Texture,
    TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::{runtime::BinTexture, TextureFormat};

use crate::core::RenderCommandBuilder;
use crate::core::UploadTextureCommand;
use crate::{
    components::{LightComponent, VisualComponent},
    core::{GpuUploadManager, TransferError, UploadGPUResource, UploadGPUTexture},
};

use super::RenderModel;
use super::{PersistentDescriptorSetManager, TextureSlot, MISSING_MODEL_RESOURCE_ID};

macro_rules! declare_texture_resource_id {
    ($name:ident, $uuid:expr) => {
        #[allow(unsafe_code)]
        pub const $name: ResourceTypeAndId = ResourceTypeAndId {
            kind: lgn_graphics_data::runtime::BinTexture::TYPE,
            id: ResourceId::from_uuid(uuid!($uuid)),
        };
    };
}

declare_texture_resource_id!(ALBEDO_RESOURCE_ID, "96f640c3-adc1-4a00-9114-9a5fc5a44d55");

declare_texture_resource_id!(NORMAL_RESOURCE_ID, "1ca43c76-817e-4286-8f21-b08087b458d5");

declare_texture_resource_id!(
    METALNESS_RESOURCE_ID,
    "50fc31f2-454b-4e6a-9c4b-cb0b5783d7f4"
);

declare_texture_resource_id!(
    ROUGHNESS_RESOURCE_ID,
    "756e99f7-e94d-4744-ada0-c84880f7da74"
);

pub const DEFAULT_TEXTURE_RESOURCE_IDS: [ResourceTypeAndId; DefaultTextureId::COUNT] = [
    ALBEDO_RESOURCE_ID,
    NORMAL_RESOURCE_ID,
    METALNESS_RESOURCE_ID,
    ROUGHNESS_RESOURCE_ID,
];

#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter)]
pub enum DefaultTextureId {
    Albedo,
    Normal,
    Metalness,
    Roughness,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum TextureManagerError {
    #[error(transparent)]
    TransferError(#[from] TransferError),
}

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
        Self {
            data: Arc::new(
                mips_data
                    .iter()
                    .map(|x| Self::to_vec_u8(x))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn mips(&self) -> &[Vec<u8>] {
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

impl From<BinTexture> for TextureData {
    fn from(mut bin_texture: BinTexture) -> Self {
        Self {
            data: Arc::new(
                bin_texture
                    .mips
                    .drain(..)
                    .map(|mip| mip.texel_data.into_vec())
                    .collect(),
            ),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct RenderTexture {
    data: TextureData,
    gpu_texture: Texture,
    default_gpu_view: TextureView,
    bindless_slot: TextureSlot,
}
lgn_data_runtime::implement_runtime_resource!(RenderTexture);

#[allow(dead_code)]
impl RenderTexture {
    pub fn data(&self) -> &TextureData {
        &self.data
    }

    pub fn gpu_texture(&self) -> &Texture {
        &self.gpu_texture
    }

    pub fn bindless_slot(&self) -> TextureSlot {
        self.bindless_slot
    }
}

impl Drop for RenderTexture {
    fn drop(&mut self) {}
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
        asset_registry: &AssetRegistry,
        component: &dyn lgn_data_runtime::Component,
        entity_command: &mut EntityCommands<'_, '_, '_>,
    ) -> Result<(), AssetRegistryError> {
        // Visual Test

        if let Some(visual) = component.downcast_ref::<lgn_graphics_data::runtime::Visual>() {
            // The data might not contain a valid resource ID but we set a default model at runtime in order to visualize the visual.
            let model_resource_id = visual
                .renderable_geometry
                .as_ref()
                .map_or(MISSING_MODEL_RESOURCE_ID, |r| r.id());

            let render_model_handle = asset_registry
                .lookup::<RenderModel>(&model_resource_id)
                .expect("Must be loaded");

            entity_command.insert(VisualComponent::new(
                &render_model_handle,
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

impl Drop for TextureInstaller {
    fn drop(&mut self) {}
}

#[async_trait]
impl ResourceInstaller for TextureInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        _request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<Box<dyn Resource>, AssetRegistryError> {
        let texture_data =
            from_binary_reader::<lgn_graphics_data::runtime::BinTexture>(reader).await?;
        lgn_tracing::info!(
            "Texture {} | width: {}, height: {}, format: {:?}",
            resource_id.id,
            texture_data.width,
            texture_data.height,
            texture_data.format
        );

        let render_texture = self
            .texture_manager
            .install_texture(texture_data, &resource_id.to_string())
            .await
            .map_err(|x| AssetRegistryError::Generic(x.to_string()))?;

        Ok(Box::new(render_texture))
    }
}

struct Inner {
    device_context: DeviceContext,
    persistent_descriptor_set_manager: PersistentDescriptorSetManager,
    upload_manager: GpuUploadManager,
    default_textures: Vec<RenderTexture>,
    default_texture_handles: AtomicCell<Vec<Handle<RenderTexture>>>,
}

#[derive(Clone)]
pub struct TextureManager {
    inner: Arc<Inner>,
}

impl TextureManager {
    pub fn new(
        device_context: &DeviceContext,
        render_commands: &mut RenderCommandBuilder,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
        upload_manager: &GpuUploadManager,
    ) -> Self {
        let default_textures = DefaultTextureId::iter()
            .map(|shared_texture_id| {
                let (texture_def, texture_data, name) = match shared_texture_id {
                    DefaultTextureId::Albedo => Self::create_albedo_texture(),
                    DefaultTextureId::Normal => Self::create_normal_texture(),
                    DefaultTextureId::Metalness => Self::create_metalness_texture(),
                    DefaultTextureId::Roughness => Self::create_roughness_texture(),
                };

                let texture = device_context.create_texture(texture_def, &name);
                let texture_view = texture.create_view(TextureViewDef::as_shader_resource_view(
                    texture.definition(),
                ));

                render_commands.push(UploadTextureCommand {
                    src_data: texture_data.clone(),
                    dst_texture: texture.clone(),
                });

                RenderTexture {
                    bindless_slot: persistent_descriptor_set_manager
                        .allocate_texture_slot(&texture_view),
                    data: texture_data,
                    gpu_texture: texture,
                    default_gpu_view: texture_view,
                }
            })
            .collect::<Vec<_>>();

        Self {
            inner: Arc::new(Inner {
                device_context: device_context.clone(),
                persistent_descriptor_set_manager: persistent_descriptor_set_manager.clone(),
                upload_manager: upload_manager.clone(),
                default_textures,
                default_texture_handles: AtomicCell::new(Vec::new()),
            }),
        }
    }

    pub fn get_default_texture(&self, default_texture_id: DefaultTextureId) -> &RenderTexture {
        &self.inner.default_textures[default_texture_id as usize]
    }

    pub fn install_default_resources(&self, asset_registry: &AssetRegistry) {
        let mut default_texture_handles = Vec::with_capacity(DefaultTextureId::COUNT);
        DefaultTextureId::iter()
            .enumerate()
            .for_each(|(index, default_texture_type)| {
                let handle = asset_registry
                    .set_resource(
                        DEFAULT_TEXTURE_RESOURCE_IDS[index],
                        Box::new(self.get_default_texture(default_texture_type).clone()),
                    )
                    .unwrap();
                default_texture_handles.push(Handle::<RenderTexture>::from(handle));
            });
        self.inner
            .default_texture_handles
            .store(default_texture_handles);
    }

    async fn install_texture(
        &self,
        bin_texture: BinTexture,
        name: &str,
    ) -> Result<RenderTexture, TextureManagerError> {
        let texture_def = Self::texture_def_from_data(&bin_texture);
        let gpu_texture = self.inner.device_context.create_texture(texture_def, name);
        let default_gpu_view =
            gpu_texture.create_view(TextureViewDef::as_shader_resource_view(&texture_def));
        let bindless_slot = self
            .inner
            .persistent_descriptor_set_manager
            .allocate_texture_slot(&default_gpu_view);
        let texture_data = TextureData::from(bin_texture);
        self.inner
            .upload_manager
            .async_upload(UploadGPUResource::Texture(UploadGPUTexture {
                src_data: texture_data.clone(),
                dst_texture: gpu_texture.clone(),
            }))?
            .await?;

        Ok(RenderTexture {
            data: texture_data,
            gpu_texture,
            default_gpu_view,
            bindless_slot,
        })
    }

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

    fn create_albedo_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_SRGB,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        // https://colorpicker.me/#9b0eab
        texture_data[0] = Color::new(155, 14, 171, 255);
        texture_data[1] = Color::new(155, 14, 171, 255);
        texture_data[2] = Color::new(155, 14, 171, 255);
        texture_data[3] = Color::new(155, 14, 171, 255);

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "default_albedo".to_string(),
        )
    }

    fn create_normal_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        texture_data[0] = Color::new(127, 127, 255, 255);
        texture_data[1] = Color::new(127, 127, 255, 255);
        texture_data[2] = Color::new(127, 127, 255, 255);
        texture_data[3] = Color::new(127, 127, 255, 255);

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "default_normal".to_string(),
        )
    }

    fn create_metalness_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 0;
        texture_data[1] = 0;
        texture_data[2] = 0;
        texture_data[3] = 0;

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "Metalness".to_string(),
        )
    }

    fn create_roughness_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 240;
        texture_data[1] = 240;
        texture_data[2] = 240;
        texture_data[3] = 240;

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "Roughness".to_string(),
        )
    }
}
