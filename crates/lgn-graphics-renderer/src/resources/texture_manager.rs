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

use crate::{
    components::{LightComponent, VisualComponent},
    core::{GpuUploadManager, TransferError, UploadGPUResource, UploadGPUTexture},
};

use super::{PersistentDescriptorSetManager, TextureSlot};

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

struct Inner {
    device_context: DeviceContext,
    persistent_descriptor_set_manager: PersistentDescriptorSetManager,
    upload_manager: GpuUploadManager,
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
    data: TextureData,
    gpu_texture: Texture,
    default_gpu_view: TextureView,
    bindless_slot: TextureSlot,
}
lgn_data_runtime::implement_runtime_resource!(RenderTexture);

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
    fn drop(&mut self) {
        todo!()
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
            .async_create_texture(texture_data, &resource_id.to_string())
            .await
            .map_err(|x| AssetRegistryError::Generic(x.to_string()))?;

        Ok(Box::new(render_texture))
    }
}

impl TextureManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
        upload_manager: &GpuUploadManager,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                device_context: device_context.clone(),
                persistent_descriptor_set_manager: persistent_descriptor_set_manager.clone(),
                upload_manager: upload_manager.clone(),
            }),
        }
    }

    async fn async_create_texture(
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
}
