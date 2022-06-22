use std::sync::Arc;

use lgn_data_runtime::{from_binary_reader, prelude::*};

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

impl From<u32> for MaterialId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct RenderMaterial {
    material_id: MaterialId,
    va: u64,
}
lgn_data_runtime::implement_runtime_resource!(RenderMaterial);

impl Drop for RenderMaterial {
    fn drop(&mut self) {
        todo!()
    }
}

impl RenderMaterial {
    pub fn bindless_slot(&self) -> MaterialId {
        self.material_id
    }

    pub fn va(&self) -> u64 {
        self.va
    }
}

pub(crate) struct MaterialInstaller {
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
        let material_data =
            from_binary_reader::<lgn_graphics_data::runtime::Material>(reader).await?;

        lgn_tracing::info!("Material {}", resource_id.id,);

        let render_material = self
            .material_manager
            .async_create_material(
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
            GpuMaterialDataManager::new(gpu_heap, MATERIAL_BLOCK_SIZE, &gpu_upload_manager);

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

    // pub fn get_material_id_from_resource_id(
    //     &self,
    //     resource_id: &ResourceTypeAndId,
    // ) -> Option<MaterialId> {
    //     self.inner
    //         .resource_id_to_material_id
    //         .get(resource_id)
    //         .copied()
    // }

    // pub fn get_material_id_from_resource_id_unchecked(
    //     &self,
    //     resource_id: &ResourceTypeAndId,
    // ) -> MaterialId {
    //     *self
    //         .inner
    //         .resource_id_to_material_id
    //         .get(resource_id)
    //         .unwrap()
    // }

    // pub fn is_material_ready(&self, material_id: MaterialId) -> bool {
    //     let slot = &self.inner.materials[material_id.index() as usize];
    //     match slot {
    //         MaterialSlot::Empty => panic!("Invalid material id"),
    //         MaterialSlot::Occupied(_) => true,
    //     }
    // }

    // pub fn get_material(&self, material_id: MaterialId) -> &Material {
    //     let slot = &self.inner.materials[material_id.index() as usize];
    //     match slot {
    //         MaterialSlot::Empty => panic!("Invalid material id"),
    //         MaterialSlot::Occupied(material) => material,
    //     }
    // }

    // fn add_material(&mut self, entity: Entity, material_component: &MaterialComponent) {
    //     let material_resource_id = material_component.resource.id();
    //     let material_id: MaterialId = self.inner.index_allocator.allocate().into();
    //     let material_data = material_component.material_data.clone();

    //     self.alloc_material(material_id, material_resource_id, material_data);

    //     self.inner.upload_queue.insert(material_id);

    //     self.inner
    //         .resource_id_to_material_id
    //         .insert(material_resource_id, material_id);

    //     self.inner.entity_to_material_id.insert(entity, material_id);

    //     self.inner.material_id_to_texture_ids.insert(
    //         material_id,
    //         Self::collect_texture_dependencies(&material_component.material_data),
    //     );
    // }

    // fn collect_texture_dependencies(material_data: &MaterialData) -> Vec<ResourceTypeAndId> {
    //     let mut result = Vec::new();

    //     if material_data.albedo_texture.is_some() {
    //         result.push(material_data.albedo_texture.as_ref().unwrap().id());
    //     }
    //     if material_data.normal_texture.is_some() {
    //         result.push(material_data.normal_texture.as_ref().unwrap().id());
    //     }
    //     if material_data.metalness_texture.is_some() {
    //         result.push(material_data.metalness_texture.as_ref().unwrap().id());
    //     }
    //     if material_data.roughness_texture.is_some() {
    //         result.push(material_data.roughness_texture.as_ref().unwrap().id());
    //     }

    //     result
    // }

    async fn build_gpu_material_data(
        asset_registry: &AssetRegistry,
        material_component: &lgn_graphics_data::runtime::Material,
        shared_resources_manager: &SharedResourcesManager,
        sampler_manager: &SamplerManager,
    ) -> Result<crate::cgen::cgen_type::MaterialData, MaterialManagerError> {
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
            Self::get_texture_slot(
                asset_registry,
                material_component.albedo.as_ref(),
                SharedTextureId::Albedo,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        material_data.set_normal_texture(
            Self::get_texture_slot(
                asset_registry,
                material_component.normal.as_ref(),
                SharedTextureId::Normal,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        material_data.set_metalness_texture(
            Self::get_texture_slot(
                asset_registry,
                material_component.metalness.as_ref(),
                SharedTextureId::Metalness,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        material_data.set_roughness_texture(
            Self::get_texture_slot(
                asset_registry,
                material_component.roughness.as_ref(),
                SharedTextureId::Roughness,
                shared_resources_manager,
            )
            .await?
            .index()
            .into(),
        );
        material_data.set_sampler(
            Self::get_sampler_slot(
                sampler_manager,
                material_component.sampler.as_ref(),
                shared_resources_manager,
            )
            .index()
            .into(),
        );

        Ok(material_data)
    }

    async fn get_texture_slot(
        asset_registry: &AssetRegistry,
        texture_id: Option<&BinTextureReferenceType>,
        default_shared_id: SharedTextureId,
        shared_resources_manager: &SharedResourcesManager,
    ) -> Result<TextureSlot, AssetRegistryError> {
        let texture_slot = if let Some(texture_id) = texture_id {
            let render_texture = asset_registry
                .load_async::<RenderTexture>(texture_id.id())
                .await?;
            let render_texture = render_texture.get().unwrap();
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

    // fn upload_material_data(
    //     &mut self,
    //     renderer: &Renderer,
    //     shared_resources_manager: &SharedResourcesManager,
    //     missing_visuals_tracker: &mut MissingVisualTracker,
    //     sampler_manager: &SamplerManager,
    //     asset_registry: &Arc<AssetRegistry>,
    // ) {
    //     let mut render_commands = renderer.render_command_builder();

    //     for material_id in &self.inner.upload_queue {
    //         let material = &self.get_material(*material_id);
    //         let material_data = &material.material_data;

    //         let gpu_material_data = Self::build_gpu_material_data(
    //             asset_registry,
    //             material_data,
    //             shared_resources_manager,
    //             sampler_manager,
    //         );
    //         self.inner.gpu_material_data.update_gpu_data(
    //             material_id,
    //             &gpu_material_data,
    //             &mut render_commands,
    //         );

    //         // TODO(vdbdd): remove asap
    //         missing_visuals_tracker.add_changed_resource(material.resource_id);
    //     }

    //     self.inner.upload_queue.clear();
    // }

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

        let default_material_id = index_allocator.allocate().into();
        let gpu_data_allocation = gpu_material_data_manager.alloc_gpu_data(&default_material_id);

        gpu_material_data_manager.update_gpu_data(
            &default_material_id,
            &default_material_data,
            render_commands,
        );

        RenderMaterial {
            material_id: default_material_id,
            va: gpu_data_allocation.va_address(),
        }
    }

    async fn async_create_material(
        &self,
        asset_registry: &Arc<AssetRegistry>,
        material_data: &lgn_graphics_data::runtime::Material,
        _name: &str,
    ) -> Result<RenderMaterial, MaterialManagerError> {
        let gpu_material_data = Self::build_gpu_material_data(
            asset_registry.as_ref(),
            &material_data,
            &self.inner.shared_resources_manager,
            &self.inner.sampler_manager,
        )
        .await?;

        let material_id = {
            let mut index_allocator = self.inner.index_allocator.write();
            index_allocator.allocate().into()
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
            va: gpu_data_allocation.va_address(),
        })
    }

    fn create_material(
        &mut self,
        material_id: MaterialId,
        material_data: lgn_graphics_data::runtime::Material,
    ) {
        panic!();
        // if material_id.index() as usize >= self.inner.materials.len() {
        //     let next_size =
        //         round_size_up_to_alignment_u32(material_id.index() + 1, MATERIAL_BLOCK_SIZE);
        //     self.inner
        //         .materials
        //         .resize(next_size as usize, MaterialSlot::Empty);
        // }
        // self.inner.gpu_material_data.alloc_gpu_data(&material_id);
        // self.inner.materials[material_id.index() as usize] = MaterialSlot::Occupied(Material {
        //     va: self.inner.gpu_material_data.va_for_key(&material_id),
        //     resource_id,
        //     material_data,
        // });
    }
}

// #[allow(clippy::needless_pass_by_value)]
// fn on_material_added(
//     mut commands: Commands<'_, '_>,
//     renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
//     query: Query<'_, '_, (Entity, &MaterialComponent), Added<MaterialComponent>>,
// ) {
//     let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
//     for (entity, material_component) in query.iter() {
//         material_manager.add_material(entity, material_component);

//         commands
//             .entity(entity)
//             .insert(GPUMaterialComponent::default());
//     }
// }

// #[allow(clippy::needless_pass_by_value)]
// fn upload_default_material(
//     renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
//     shared_resources_manager: Res<'_, SharedResourcesManager>,
// ) {
//     let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
//     material_manager.upload_default_material(&renderer, &shared_resources_manager);
// }

// #[allow(clippy::needless_pass_by_value)]
// fn upload_material_data(
//     renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
//     shared_resources_manager: Res<'_, SharedResourcesManager>,
//     asset_registry: Res<'_, Arc<AssetRegistry>>,
// ) {
//     let mut material_manager = renderer.render_resources().get_mut::<MaterialManager>();
//     let mut missing_visuals_tracker = renderer
//         .render_resources()
//         .get_mut::<MissingVisualTracker>();

//     let sampler_manager = renderer.render_resources().get::<SamplerManager>();
//     material_manager.upload_material_data(
//         &renderer,
//         &shared_resources_manager,
//         &mut missing_visuals_tracker,
//         &sampler_manager,
//         &asset_registry,
//     );
// }
