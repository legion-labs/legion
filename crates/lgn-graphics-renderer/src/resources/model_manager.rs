use std::sync::Arc;

use async_trait::async_trait;

use crossbeam::atomic::AtomicCell;
use lgn_data_runtime::{
    activate_reference, from_binary_reader, AssetRegistry, AssetRegistryError, AssetRegistryReader,
    Handle, LoadRequest, Resource, ResourceDescriptor, ResourceId, ResourceInstaller,
    ResourceTypeAndId,
};

use strum::{EnumCount, IntoEnumIterator};

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
            id: unsafe { ResourceId::from_raw_unchecked(u128::from_le_bytes($uuid)) },
        };
    };
}

declare_model_resource_id!(
    PLANE_MODEL_RESOURCE_ID,
    [
        0x36, 0x0F, 0xCA, 0x12, 0xBC, 0xFC, 0x43, 0xB1, 0xA3, 0x98, 0xFC, 0xB6, 0x05, 0xBD, 0x6A,
        0x95,
    ]
);

declare_model_resource_id!(
    CUBE_MODEL_RESOURCE_ID,
    [
        0x43, 0x39, 0x8C, 0x28, 0x7B, 0x16, 0x45, 0xB3, 0x8C, 0x61, 0xCD, 0xBF, 0x84, 0xC6, 0x32,
        0xFA
    ]
);

declare_model_resource_id!(
    PYRAMID_MODEL_RESOURCE_ID,
    [
        0x2D, 0x63, 0x0F, 0x4D, 0xD7, 0x87, 0x49, 0x13, 0x9A, 0xE8, 0x8F, 0x43, 0xAC, 0x78, 0xCC,
        0x02
    ]
);

declare_model_resource_id!(
    WIREFRAMECUBE_MODEL_RESOURCE_ID,
    [
        0x94, 0x88, 0x00, 0xAB, 0x14, 0x82, 0x42, 0xC4, 0x84, 0x58, 0xEC, 0xD2, 0x60, 0x7A, 0xB6,
        0x1A
    ]
);

declare_model_resource_id!(
    GROUNDPLANE_MODEL_RESOURCE_ID,
    [
        0xF6, 0x01, 0x17, 0xEF, 0x1D, 0xDE, 0x4F, 0xD1, 0xAD, 0x8A, 0xD3, 0xDF, 0xFC, 0x9A, 0x2E,
        0x6A
    ]
);

declare_model_resource_id!(
    TORUS_MODEL_RESOURCE_ID,
    [
        0x76, 0x87, 0x8B, 0x61, 0xFC, 0xC5, 0x43, 0x83, 0xBA, 0xC4, 0x73, 0xF1, 0xB5, 0x50, 0x5A,
        0x39
    ]
);

declare_model_resource_id!(
    CONE_MODEL_RESOURCE_ID,
    [
        0xA3, 0x4A, 0xDE, 0xB0, 0x5C, 0x39, 0x42, 0x6A, 0xB0, 0x08, 0xD0, 0x3A, 0x2B, 0xCC, 0x09,
        0xE8
    ]
);

declare_model_resource_id!(
    CYLINDER_MODEL_RESOURCE_ID,
    [
        0x6E, 0x80, 0x18, 0xDD, 0x88, 0xE4, 0x4E, 0x9D, 0xA8, 0xB3, 0x94, 0x4F, 0xC7, 0xC6, 0x0E,
        0xDC
    ]
);

declare_model_resource_id!(
    SPHERE_MODEL_RESOURCE_ID,
    [
        0xC5, 0xE6, 0x56, 0x71, 0x74, 0x6E, 0x4B, 0xA2, 0xAE, 0x19, 0x8C, 0xB5, 0x70, 0x82, 0x16,
        0x90,
    ]
);

declare_model_resource_id!(
    ARROW_MODEL_RESOURCE_ID,
    [
        0x07, 0x00, 0x7A, 0x8A, 0x8C, 0x16, 0x41, 0x77, 0x89, 0x5A, 0x74, 0x9B, 0x6F, 0x7A, 0x08,
        0x01
    ]
);

declare_model_resource_id!(
    ROTATIONRING_MODEL_RESOURCE_ID,
    [
        0x9E, 0xB0, 0xB5, 0x1A, 0xC5, 0xDD, 0x45, 0x96, 0xA6, 0xD4, 0xB2, 0x06, 0xD7, 0xC5, 0x79,
        0x2E
    ]
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

// #[allow(clippy::needless_pass_by_value)]
// pub(crate) fn update_models(
//     renderer: ResMut<'_, Renderer>,
//     asset_registry: Res<'_, Arc<AssetRegistry>>,
//     updated_models: Query<'_, '_, &ModelComponent, Changed<ModelComponent>>,
// ) {
//     let mut mesh_manager = renderer.render_resources().get_mut::<MeshManager>();
//     let mut model_manager = renderer.render_resources().get_mut::<ModelManager>();
//     let material_manager = renderer.render_resources().get::<MaterialManager>();
//     let mut missing_visuals_tracker = renderer
//         .render_resources()
//         .get_mut::<MissingVisualTracker>();

//     let mut render_commands = renderer.render_command_builder();

//     for updated_model in updated_models.iter() {
//         let model_resource_id = &updated_model.resource.id();

//         missing_visuals_tracker.add_changed_resource(*model_resource_id);

//         let mut mesh_instances = Vec::new();

//         for mesh in &updated_model.meshes {
//             let mesh_id = mesh_manager.add_mesh(&mut render_commands, mesh);

//             let render_material = if let Some(material_resource_id) = &mesh.material_id {
//                 let render_material_guard = asset_registry
//                     .lookup::<RenderMaterial>(&material_resource_id.id())
//                     .expect("Must be installed");

//                 let render_material = render_material_guard.get().unwrap().clone();

//                 render_material
//             } else {
//                 material_manager.get_default_material().clone()
//             };

//             mesh_instances.push(MeshInstance {
//                 mesh_id,
//                 material_id: render_material.material_id(),
//                 material_va: render_material.gpuheap_addr(),
//             });
//         }

//         model_manager.add_model(*model_resource_id, ModelMetaData { mesh_instances });
//     }
// }

// #[allow(clippy::needless_pass_by_value)]
// fn debug_bounding_spheres(
//     debug_display: Res<'_, DebugDisplay>,
//     bump_allocator_pool: Res<'_, BumpAllocatorPool>,
//     renderer: Res<'_, Renderer>,
//     renderer_options: Res<'_, RendererOptions>,
//     visuals: Query<'_, '_, (&VisualComponent, &Transform)>,
// ) {
//     let mesh_manager = renderer.render_resources().get_mut::<MeshManager>();
//     let model_manager = renderer.render_resources().get_mut::<ModelManager>();

//     if !renderer_options.show_bounding_spheres {
//         return;
//     }

//     bump_allocator_pool.scoped_bump(|bump| {
//         debug_display.create_display_list(bump, |builder| {
//             for (visual, transform) in visuals.iter() {
//                 if let Some(model_resource_id) = visual.model_resource_id() {
//                     if let Some(model) = model_manager.get_model_meta_data(model_resource_id) {
//                         for mesh in &model.mesh_instances {
//                             let mesh_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
//                             builder.add_default_mesh(
//                                 &GlobalTransform::identity()
//                                     .with_translation(
//                                         transform.translation
//                                             + mesh_data.bounding_sphere.truncate(),
//                                     )
//                                     .with_scale(
//                                         Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w,
//                                     )
//                                     .with_rotation(transform.rotation),
//                                 DefaultMeshType::Sphere,
//                                 Color::WHITE,
//                             );
//                         }
//                     }
//                 } else {
//                     let model = model_manager.get_default_model(DefaultMeshType::Cube);
//                     for mesh in &model.mesh_instances {
//                         let mesh_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
//                         builder.add_default_mesh(
//                             &GlobalTransform::identity()
//                                 .with_translation(
//                                     transform.translation + mesh_data.bounding_sphere.truncate(),
//                                 )
//                                 .with_scale(Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w)
//                                 .with_rotation(transform.rotation),
//                             DefaultMeshType::Sphere,
//                             Color::WHITE,
//                         );
//                     }
//                 }
//             }
//         });
//     });
// }
