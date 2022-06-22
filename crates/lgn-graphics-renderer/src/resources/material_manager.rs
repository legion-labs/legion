use std::sync::Arc;

use lgn_data_runtime::{activate_reference, from_binary_reader, prelude::*};

use lgn_graphics_api::{AddressMode, CompareOp, FilterType, MipMapMode, SamplerDef};
use lgn_graphics_data::{runtime::BinTextureReferenceType, runtime::SamplerData};
use lgn_math::Vec4;

use crate::{
    core::{GpuUploadManager, RenderCommandBuilder, TransferError},
    resources::SharedTextureId,
};

use super::{
    GpuDataManager, IndexAllocator, RenderTexture, SamplerManager, SamplerSlot,
    SharedResourcesManager, TextureSlot, UnifiedStaticBuffer,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum MaterialManagerError {
    #[error(transparent)]
    AssetRegistryError(#[from] AssetRegistryError),

    #[error(transparent)]
    TransferError(#[from] TransferError),
}

type GpuMaterialDataManager = GpuDataManager<MaterialId, crate::cgen::cgen_type::MaterialData>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(u32);

impl MaterialId {
    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone)]
pub struct RenderMaterial {
    material_id: MaterialId,
    gpuheap_addr: u64,
}
lgn_data_runtime::implement_runtime_resource!(RenderMaterial);

impl RenderMaterial {
    pub fn material_id(&self) -> MaterialId {
        self.material_id
    }

    pub fn gpuheap_addr(&self) -> u64 {
        self.gpuheap_addr
    }
}

impl Drop for RenderMaterial {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct MaterialInstaller {
    material_manager: MaterialManager,
}

impl MaterialInstaller {
    pub(crate) fn new(material_manager: &MaterialManager) -> Self {
        Self {
            material_manager: material_manager.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ResourceInstaller for MaterialInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<Box<dyn Resource>, AssetRegistryError> {
        let mut material_data =
            from_binary_reader::<lgn_graphics_data::runtime::Material>(reader).await?;

        lgn_tracing::info!("Material {}", resource_id.id,);

        activate_reference(
            resource_id,
            &mut material_data,
            request.asset_registry.clone(),
        )
        .await;

        let render_material = self
            .material_manager
            .install_material(
                &request.asset_registry,
                &material_data,
                &resource_id.to_string(),
            )
            .await
            .map_err(|x| AssetRegistryError::Generic(x.to_string()))?;

        Ok(Box::new(render_material))
    }
}

const MATERIAL_BLOCK_SIZE: u32 = 2048;

struct Inner {
    shared_resources_manager: SharedResourcesManager,
    sampler_manager: SamplerManager,
    index_allocator: parking_lot::RwLock<IndexAllocator>,
    gpu_material_data_manager: tokio::sync::RwLock<GpuMaterialDataManager>,
    default_material: RenderMaterial,
}

#[derive(Clone)]
pub struct MaterialManager {
    inner: Arc<Inner>,
}

impl MaterialManager {
    pub fn new(
        gpu_heap: &UnifiedStaticBuffer,
        gpu_upload_manager: &GpuUploadManager,
        render_commands: &mut RenderCommandBuilder,
        shared_resources_manager: &SharedResourcesManager,
        sampler_manager: &SamplerManager,
    ) -> Self {
        let mut index_allocator = IndexAllocator::new(MATERIAL_BLOCK_SIZE);

        let mut gpu_material_data_manager =
            GpuMaterialDataManager::new(gpu_heap, MATERIAL_BLOCK_SIZE, gpu_upload_manager);

        let default_material = Self::install_default_material(
            &mut index_allocator,
            &mut gpu_material_data_manager,
            render_commands,
            shared_resources_manager,
        );

        Self {
            inner: Arc::new(Inner {
                shared_resources_manager: shared_resources_manager.clone(),
                sampler_manager: sampler_manager.clone(),
                index_allocator: parking_lot::RwLock::new(index_allocator),
                gpu_material_data_manager: tokio::sync::RwLock::new(gpu_material_data_manager),
                default_material,
            }),
        }
    }

    pub fn get_default_material(&self) -> &RenderMaterial {
        &self.inner.default_material
    }

    async fn build_gpu_data(
        asset_registry: &AssetRegistry,
        material_data: &lgn_graphics_data::runtime::Material,
        shared_resources_manager: &SharedResourcesManager,
        sampler_manager: &SamplerManager,
    ) -> Result<crate::cgen::cgen_type::MaterialData, MaterialManagerError> {
        let mut gpu_data = crate::cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(material_data.base_albedo.r) / 255.0f32,
            f32::from(material_data.base_albedo.g) / 255.0f32,
            f32::from(material_data.base_albedo.b) / 255.0f32,
            f32::from(material_data.base_albedo.a) / 255.0f32,
        );
        gpu_data.set_base_albedo(color.into());
        gpu_data.set_base_metalness(material_data.base_metalness.into());
        gpu_data.set_reflectance(material_data.reflectance.into());
        gpu_data.set_base_roughness(material_data.base_roughness.into());
        gpu_data.set_albedo_texture(
            Self::get_texture_slot(
                asset_registry,
                material_data.albedo.as_ref(),
                SharedTextureId::Albedo,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        gpu_data.set_normal_texture(
            Self::get_texture_slot(
                asset_registry,
                material_data.normal.as_ref(),
                SharedTextureId::Normal,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        gpu_data.set_metalness_texture(
            Self::get_texture_slot(
                asset_registry,
                material_data.metalness.as_ref(),
                SharedTextureId::Metalness,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        gpu_data.set_roughness_texture(
            Self::get_texture_slot(
                asset_registry,
                material_data.roughness.as_ref(),
                SharedTextureId::Roughness,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        gpu_data.set_sampler(
            Self::get_sampler_slot(
                sampler_manager,
                material_data.sampler.as_ref(),
                shared_resources_manager,
            )
            .index()
            .into(),
        );

        Ok(gpu_data)
    }

    async fn get_texture_slot(
        _asset_registry: &AssetRegistry,
        texture_id: Option<&BinTextureReferenceType>,
        default_shared_id: SharedTextureId,
        shared_resources_manager: &SharedResourcesManager,
    ) -> Result<TextureSlot, AssetRegistryError> {
        let texture_slot = if let Some(texture_id) = texture_id {
            let render_texture_handle = texture_id.get_active_handle::<RenderTexture>().unwrap();
            let render_texture = render_texture_handle.get().unwrap();
            render_texture.bindless_slot()
        } else {
            shared_resources_manager.default_texture_slot(default_shared_id)
        };
        Ok(texture_slot)
    }

    fn get_sampler_slot(
        sampler_manager: &SamplerManager,
        sampler_data: Option<&SamplerData>,
        shared_resources_manager: &SharedResourcesManager,
    ) -> SamplerSlot {
        if let Some(sampler_data) = sampler_data {
            #[allow(clippy::match_same_arms)]
            sampler_manager.get_slot(&SamplerDef {
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
            shared_resources_manager.default_sampler_slot()
        }
    }

    fn install_default_material(
        index_allocator: &mut IndexAllocator,
        gpu_material_data_manager: &mut GpuMaterialDataManager,
        render_commands: &mut RenderCommandBuilder,
        shared_resources_manager: &SharedResourcesManager,
    ) -> RenderMaterial {
        let mut default_material_data = crate::cgen::cgen_type::MaterialData::default();

        default_material_data.set_base_albedo(Vec4::new(0.8, 0.8, 0.8, 1.0).into());
        default_material_data.set_base_metalness(0.0.into());
        default_material_data.set_reflectance(0.5.into());
        default_material_data.set_base_roughness(0.4.into());
        default_material_data.set_albedo_texture(
            shared_resources_manager
                .default_texture_slot(SharedTextureId::Albedo)
                .index()
                .into(),
        );
        default_material_data.set_normal_texture(
            shared_resources_manager
                .default_texture_slot(SharedTextureId::Normal)
                .index()
                .into(),
        );
        default_material_data.set_metalness_texture(
            shared_resources_manager
                .default_texture_slot(SharedTextureId::Metalness)
                .index()
                .into(),
        );
        default_material_data.set_roughness_texture(
            shared_resources_manager
                .default_texture_slot(SharedTextureId::Roughness)
                .index()
                .into(),
        );
        default_material_data.set_sampler(
            shared_resources_manager
                .default_sampler_slot()
                .index()
                .into(),
        );

        let default_material_id = MaterialId(index_allocator.allocate());
        let gpu_data_allocation = gpu_material_data_manager.alloc_gpu_data(&default_material_id);

        gpu_material_data_manager.update_gpu_data(
            &default_material_id,
            &default_material_data,
            render_commands,
        );

        RenderMaterial {
            material_id: default_material_id,
            gpuheap_addr: gpu_data_allocation.gpuheap_addr(),
        }
    }

    async fn install_material(
        &self,
        asset_registry: &Arc<AssetRegistry>,
        material_data: &lgn_graphics_data::runtime::Material,
        _name: &str,
    ) -> Result<RenderMaterial, MaterialManagerError> {
        let gpu_material_data = Self::build_gpu_data(
            asset_registry.as_ref(),
            material_data,
            &self.inner.shared_resources_manager,
            &self.inner.sampler_manager,
        )
        .await?;

        let material_id = {
            let mut index_allocator = self.inner.index_allocator.write();
            MaterialId(index_allocator.allocate())
        };

        let gpu_data_allocation = {
            let mut gpu_material_data_manager = self.inner.gpu_material_data_manager.write().await;
            let gpu_data_allocation = gpu_material_data_manager.alloc_gpu_data(&material_id);
            gpu_material_data_manager
                .async_update_gpu_data(&material_id, &gpu_material_data)
                .await?;
            gpu_data_allocation
        };

        Ok(RenderMaterial {
            material_id,
            gpuheap_addr: gpu_data_allocation.gpuheap_addr(),
        })
    }
}
