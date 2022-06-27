use std::sync::Arc;

use async_trait::async_trait;

use crossbeam::atomic::AtomicCell;
use lgn_data_runtime::{
    activate_reference, from_binary_reader, AssetRegistry, AssetRegistryError, AssetRegistryReader,
    Handle, LoadRequest, Resource, ResourceDescriptor, ResourceId, ResourceInstaller,
    ResourceTypeAndId,
};

use strum::{EnumCount, IntoEnumIterator};

use uuid::uuid;

use crate::core::TransferError;

use super::{
    DefaultMeshType, MaterialId, MaterialManager, MeshManager, RenderMaterial, RenderMeshId,
    DEFAULT_MATERIAL_RESOURCE_ID,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum ModelManagerError {
    #[error(transparent)]
    AssetRegistryError(#[from] AssetRegistryError),

    #[error(transparent)]
    TransferError(#[from] TransferError),
}

macro_rules! declare_model_resource_id {
    ($name:ident, $uuid:expr) => {
        #[allow(unsafe_code)]
        pub const $name: ResourceTypeAndId = ResourceTypeAndId {
            kind: lgn_graphics_data::runtime::Model::TYPE,
            id: ResourceId::from_uuid(uuid!($uuid)),
        };
    };
}

declare_model_resource_id!(
    PLANE_MODEL_RESOURCE_ID,
    "7d2db6d7-dd85-4468-ad6f-ebc9eb6dab9f"
);

declare_model_resource_id!(
    CUBE_MODEL_RESOURCE_ID,
    "5a1496c5-e21c-4ccf-9f17-024574bbd545"
);

declare_model_resource_id!(
    PYRAMID_MODEL_RESOURCE_ID,
    "d66ab2e5-e2aa-41bb-8f4d-c80cfae4c61b"
);

declare_model_resource_id!(
    WIREFRAMECUBE_MODEL_RESOURCE_ID,
    "473c61b7-2882-414d-858f-f56d94b24231"
);

declare_model_resource_id!(
    GROUNDPLANE_MODEL_RESOURCE_ID,
    "dfef30e7-ab46-4e34-91d1-fe49742c08bf"
);

declare_model_resource_id!(
    TORUS_MODEL_RESOURCE_ID,
    "ce2ee4dc-c3b8-4f78-b231-584de7609551"
);

declare_model_resource_id!(
    CONE_MODEL_RESOURCE_ID,
    "507bf54a-4e20-459e-8e56-ab19d1b0c5aa"
);

declare_model_resource_id!(
    CYLINDER_MODEL_RESOURCE_ID,
    "6220657c-e087-4fcb-83c6-433ccaac02dd"
);

declare_model_resource_id!(
    SPHERE_MODEL_RESOURCE_ID,
    "908569e5-9182-4abb-bd21-a604f72dd4cb"
);

declare_model_resource_id!(
    ARROW_MODEL_RESOURCE_ID,
    "8458fe11-c253-49bf-bde5-07da4aec3878"
);

declare_model_resource_id!(
    ROTATIONRING_MODEL_RESOURCE_ID,
    "0cee2298-e0f0-4330-8849-cfffee9222dc"
);

pub const MISSING_MODEL_RESOURCE_ID: ResourceTypeAndId = CUBE_MODEL_RESOURCE_ID;

pub const DEFAULT_MODEL_RESOURCE_IDS: [ResourceTypeAndId; DefaultMeshType::COUNT] = [
    PLANE_MODEL_RESOURCE_ID,
    CUBE_MODEL_RESOURCE_ID,
    PYRAMID_MODEL_RESOURCE_ID,
    WIREFRAMECUBE_MODEL_RESOURCE_ID,
    GROUNDPLANE_MODEL_RESOURCE_ID,
    TORUS_MODEL_RESOURCE_ID,
    CONE_MODEL_RESOURCE_ID,
    CYLINDER_MODEL_RESOURCE_ID,
    SPHERE_MODEL_RESOURCE_ID,
    ARROW_MODEL_RESOURCE_ID,
    ROTATIONRING_MODEL_RESOURCE_ID,
];

struct RenderModelInner {
    mesh_instances: Vec<MeshInstance>,
}

#[derive(Clone)]
pub struct RenderModel {
    inner: Arc<RenderModelInner>,
}
lgn_data_runtime::implement_runtime_resource!(RenderModel);

impl RenderModel {
    pub fn mesh_instances(&self) -> &[MeshInstance] {
        &self.inner.mesh_instances
    }
}

pub struct MeshInstance {
    pub mesh_id: RenderMeshId,
    pub material_id: MaterialId,
    pub material_va: u64,
}

pub struct ModelInstaller {
    model_manager: ModelManager,
}

impl ModelInstaller {
    pub fn new(model_manager: &ModelManager) -> Self {
        Self {
            model_manager: model_manager.clone(),
        }
    }
}

#[async_trait]
impl ResourceInstaller for ModelInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<Box<dyn Resource>, AssetRegistryError> {
        let mut model_data =
            from_binary_reader::<lgn_graphics_data::runtime::Model>(reader).await?;

        lgn_tracing::info!(
            "Model {} | ({} meshes)",
            resource_id.id,
            model_data.meshes.len(),
        );

        activate_reference(resource_id, &mut model_data, request.asset_registry.clone()).await;

        let render_model = self
            .model_manager
            .install_model(
                &request.asset_registry,
                model_data,
                &resource_id.to_string(),
            )
            .await
            .map_err(|x| AssetRegistryError::Generic(x.to_string()))?;

        Ok(Box::new(render_model))
    }
}

struct Inner {
    mesh_manager: MeshManager,
    default_models: Vec<RenderModel>,
    default_model_handles: AtomicCell<Vec<Handle<RenderModel>>>,
}

#[derive(Clone)]
pub struct ModelManager {
    inner: Arc<Inner>,
}

impl ModelManager {
    pub fn new(mesh_manager: &MeshManager, material_manager: &MaterialManager) -> Self {
        let default_material = material_manager.get_default_material();
        let mut default_models = Vec::new();

        for default_mesh_type in DefaultMeshType::iter() {
            default_models.push(RenderModel {
                inner: Arc::new(RenderModelInner {
                    mesh_instances: vec![MeshInstance {
                        mesh_id: mesh_manager.get_default_mesh_id(default_mesh_type),
                        material_id: default_material.material_id(),
                        material_va: default_material.gpuheap_addr(),
                    }],
                }),
            });
        }

        Self {
            inner: Arc::new(Inner {
                mesh_manager: mesh_manager.clone(),
                default_models,
                default_model_handles: AtomicCell::new(Vec::new()),
            }),
        }
    }

    pub fn get_default_model(&self, default_mesh_type: DefaultMeshType) -> &RenderModel {
        &self.inner.default_models[default_mesh_type as usize]
    }

    pub fn install_default_resources(&self, asset_registry: &AssetRegistry) {
        let mut default_model_handles = Vec::with_capacity(DefaultMeshType::COUNT);
        DefaultMeshType::iter()
            .enumerate()
            .for_each(|(index, default_mesh_type)| {
                let handle = asset_registry
                    .set_resource(
                        DEFAULT_MODEL_RESOURCE_IDS[index],
                        Box::new(self.get_default_model(default_mesh_type).clone()),
                    )
                    .unwrap();
                default_model_handles.push(Handle::<RenderModel>::from(handle));
            });
        self.inner
            .default_model_handles
            .store(default_model_handles);
    }

    pub async fn install_model(
        &self,
        asset_registry: &AssetRegistry,
        model_data: lgn_graphics_data::runtime::Model,
        _name: &str,
    ) -> Result<RenderModel, ModelManagerError> {
        let mut mesh_instances = Vec::new();
        for mesh_data in &model_data.meshes {
            let material_resource_id = mesh_data
                .material
                .as_ref()
                .map_or(DEFAULT_MATERIAL_RESOURCE_ID, |x| x.id());
            let mesh = mesh_data.into();
            let mesh_id = self.inner.mesh_manager.install_mesh(mesh).await?;
            let render_material_handle = asset_registry
                .lookup::<RenderMaterial>(&material_resource_id)
                .expect("Must be loaded");
            let render_material = render_material_handle.get().unwrap();

            mesh_instances.push(MeshInstance {
                mesh_id,
                material_id: render_material.material_id(),
                material_va: render_material.gpuheap_addr(),
            });
        }

        Ok(RenderModel {
            inner: Arc::new(RenderModelInner { mesh_instances }),
        })
    }
}
