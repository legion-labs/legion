use std::collections::BTreeMap;

use async_trait::async_trait;
use lgn_app::App;
use lgn_core::BumpAllocatorPool;
use lgn_data_runtime::{
    from_binary_reader, AssetRegistryError, AssetRegistryReader, HandleUntyped, LoadRequest,
    ResourceDescriptor, ResourceId, ResourceInstaller, ResourceTypeAndId,
};
use lgn_ecs::{
    prelude::{Changed, Query, Res, ResMut},
    schedule::SystemSet,
};
use lgn_graphics_data::Color;
use lgn_math::Vec3;
use lgn_tracing::warn;
use lgn_transform::components::{GlobalTransform, Transform};
use strum::IntoEnumIterator;

use crate::{
    components::{ModelComponent, VisualComponent},
    debug_display::DebugDisplay,
    labels::RenderStage,
    Renderer, ResourceStageLabel,
};

use super::{
    DefaultMeshType, MaterialId, MaterialManager, MeshId, MeshManager, MissingVisualTracker,
    RendererOptions,
};
pub struct MeshInstance {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
}

pub struct ModelMetaData {
    pub mesh_instances: Vec<MeshInstance>,
}

pub(crate) struct ModelInstaller {}

impl ModelInstaller {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ResourceInstaller for ModelInstaller {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let model = from_binary_reader::<lgn_graphics_data::runtime::Model>(reader).await?;
        lgn_tracing::info!("Model {} | ({} meshes)", resource_id.id, model.meshes.len(),);

        /*
           let model = resource.get(asset_registry)?;

            let mut entity = if let Some(entity) = asset_to_entity_map.get(resource.id()) {
                commands.entity(entity)
            } else {
                commands.spawn()
            };

            let mut meshes = Vec::new();
            for mesh in &model.meshes {
                assert!(!mesh.indices.is_empty());
                assert!(!mesh.positions.is_empty());
                assert_eq!(mesh.indices.len() % 3, 0); // triangle list
                meshes.push(Mesh {
                    indices: mesh.indices.clone(),
                    positions: mesh.positions.clone(),
                    normals: if !mesh.normals.is_empty() {
                        Some(mesh.normals.clone())
                    } else {
                        None
                    },
                    tangents: if !mesh.tangents.is_empty() {
                        Some(mesh.tangents.clone())
                    } else {
                        None
                    },
                    tex_coords: if !mesh.tex_coords.is_empty() {
                        Some(mesh.tex_coords.clone())
                    } else {
                        None
                    },
                    colors: if !mesh.colors.is_empty() {
                        Some(mesh.colors.iter().map(|v| Into::into(*v)).collect())
                    } else {
                        None
                    },
                    material_id: mesh.material.clone(),
                    bounding_sphere: Mesh::calculate_bounding_sphere(&mesh.positions),
                    topology: MeshTopology::TriangleList,
                });
            }

            let asset_id = resource.id();

            let model_component = ModelComponent { resource, meshes };
            entity.insert(model_component);

            info!(
                "Spawned {}: {} -> ECS id: {:?}",
                asset_id.kind.as_pretty().trim_start_matches("runtime_"),
                asset_id.id,
                entity.id(),
            );
            Some(entity.id())
        */
        let handle = request.asset_registry.set_resource(resource_id, model)?;
        Ok(handle)
    }
}

pub struct ModelManager {
    model_meta_data: BTreeMap<ResourceTypeAndId, ModelMetaData>,
    default_model_ids: Vec<ResourceTypeAndId>,
}

impl ModelManager {
    pub fn new(mesh_manager: &MeshManager, material_manager: &MaterialManager) -> Self {
        let default_material_id = material_manager.get_default_material_id();

        let mut model_meta_data = BTreeMap::new();

        let default_model_ids = DefaultMeshType::iter()
            .map(|_| ResourceTypeAndId {
                kind: lgn_graphics_data::runtime::Model::TYPE,
                id: ResourceId::new(),
            })
            .collect::<Vec<_>>();

        for default_mesh_type in DefaultMeshType::iter() {
            // TODO(vdbdd): reserved range of runtime resource id
            let resource_id = default_model_ids[default_mesh_type as usize];

            model_meta_data.insert(
                resource_id,
                ModelMetaData {
                    mesh_instances: vec![MeshInstance {
                        mesh_id: mesh_manager.get_default_mesh_id(default_mesh_type),
                        material_id: default_material_id,
                    }],
                },
            );
        }

        Self {
            model_meta_data,
            default_model_ids,
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_set_to_stage(
            RenderStage::Resource,
            SystemSet::new()
                .with_system(update_models)
                .with_system(debug_bounding_spheres)
                .label(ResourceStageLabel::Model)
                .after(ResourceStageLabel::Material),
        );
    }

    pub fn default_model_id(&self, default_mesh_type: DefaultMeshType) -> &ResourceTypeAndId {
        &self.default_model_ids[default_mesh_type as usize]
    }

    pub fn add_model(&mut self, resource_id: ResourceTypeAndId, model: ModelMetaData) {
        self.model_meta_data.insert(resource_id, model);
    }

    pub fn get_default_model(&self, default_mesh_type: DefaultMeshType) -> &ModelMetaData {
        self.get_model_meta_data(self.default_model_id(default_mesh_type))
            .unwrap()
    }

    pub fn get_model_meta_data(
        &self,
        model_resource_id: &ResourceTypeAndId,
    ) -> Option<&ModelMetaData> {
        self.model_meta_data.get(model_resource_id)
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_models(
    renderer: ResMut<'_, Renderer>, // renderer is a ResMut just to avoid concurrent accesses
    updated_models: Query<'_, '_, &ModelComponent, Changed<ModelComponent>>,
) {
    let mut mesh_manager = renderer.render_resources().get_mut::<MeshManager>();
    let mut model_manager = renderer.render_resources().get_mut::<ModelManager>();
    let material_manager = renderer.render_resources().get::<MaterialManager>();
    let mut missing_visuals_tracker = renderer
        .render_resources()
        .get_mut::<MissingVisualTracker>();

    let mut render_commands = renderer.render_command_builder();

    for updated_model in updated_models.iter() {
        let model_resource_id = &updated_model.resource.id();

        missing_visuals_tracker.add_changed_resource(*model_resource_id);

        let mut mesh_instances = Vec::new();

        for mesh in &updated_model.meshes {
            let mesh_id = mesh_manager.add_mesh(&mut render_commands, mesh);

            // for (idx, mesh) in updated_model.meshes.iter().enumerate() {
            /*
               UNCOMMENT WHEN THE RESOURCE SYSTEM IS WORKING
               A RUNTIME DEPENDENCY MUST BE LOADED AND THEN, THE MATERIAL ID MUST BE VALID

               // If there is no material set on the mesh (should not be the case until we fix that),
               // we assign the default material

               let material_id = mesh
               .material_id
               .as_ref()
               .map_or(material_manager.get_default_material_id(), |x| {
                   material_manager.get_material_id_from_resource_id(&x.id())
               });
            */

            if let Some(material_resource_id) = &mesh.material_id {
                let material_id_opt =
                    material_manager.get_material_id_from_resource_id(&material_resource_id.id());
                if material_id_opt.is_none() {
                    warn!(
                        "Dependency issue. Material {} not loaded for model {}",
                        material_resource_id.id(),
                        model_resource_id
                    );
                }
            }

            let material_id =
                mesh.material_id
                    .as_ref()
                    .map_or(material_manager.get_default_material_id(), |x| {
                        material_manager
                            .get_material_id_from_resource_id(&x.id())
                            .unwrap_or_else(|| material_manager.get_default_material_id())
                    });

            mesh_instances.push(MeshInstance {
                mesh_id,
                material_id,
            });
        }

        model_manager.add_model(*model_resource_id, ModelMetaData { mesh_instances });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn debug_bounding_spheres(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    renderer: Res<'_, Renderer>,
    renderer_options: Res<'_, RendererOptions>,
    visuals: Query<'_, '_, (&VisualComponent, &Transform)>,
) {
    let mesh_manager = renderer.render_resources().get_mut::<MeshManager>();
    let model_manager = renderer.render_resources().get_mut::<ModelManager>();

    if !renderer_options.show_bounding_spheres {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for (visual, transform) in visuals.iter() {
                if let Some(model_resource_id) = visual.model_resource_id() {
                    if let Some(model) = model_manager.get_model_meta_data(model_resource_id) {
                        for mesh in &model.mesh_instances {
                            let mesh_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                            builder.add_default_mesh(
                                &GlobalTransform::identity()
                                    .with_translation(
                                        transform.translation
                                            + mesh_data.bounding_sphere.truncate(),
                                    )
                                    .with_scale(
                                        Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w,
                                    )
                                    .with_rotation(transform.rotation),
                                DefaultMeshType::Sphere,
                                Color::WHITE,
                            );
                        }
                    }
                } else {
                    let model = model_manager.get_default_model(DefaultMeshType::Cube);
                    for mesh in &model.mesh_instances {
                        let mesh_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                        builder.add_default_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation + mesh_data.bounding_sphere.truncate(),
                                )
                                .with_scale(Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w)
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Sphere,
                            Color::WHITE,
                        );
                    }
                }
            }
        });
    });
}
