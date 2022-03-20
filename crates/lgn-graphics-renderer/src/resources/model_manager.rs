use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use lgn_app::{App, EventReader};
use lgn_core::BumpAllocatorPool;
use lgn_data_runtime::{
    AssetRegistry, AssetRegistryEvent, Resource, ResourceId, ResourceTypeAndId,
};
use lgn_ecs::prelude::{Query, Res, ResMut, Without};
use lgn_math::Vec3;
use lgn_tracing::span_fn;
use lgn_transform::components::{GlobalTransform, Transform};
use strum::IntoEnumIterator;

use crate::{
    components::{ManipulatorComponent, VisualComponent},
    debug_display::DebugDisplay,
    labels::RenderStage,
    Renderer,
};

use lgn_graphics_data::runtime::Model as ModelAsset;

use super::{
    DefaultMeshType, MaterialManager, MeshManager, MissingVisualTracker, RendererOptions,
    DEFAULT_MESH_GUIDS,
};

pub struct Mesh {
    pub mesh_id: u32,
    pub material_id: u32,
    pub material_index: u32,
}

pub struct ModelMetaData {
    pub meshes: Vec<Mesh>,
}

pub struct ModelManager {
    model_meta_datas: BTreeMap<ResourceTypeAndId, ModelMetaData>,
    default_model: ModelMetaData,
}

impl ModelManager {
    pub fn new() -> Self {
        let mut model_meta_datas = BTreeMap::new();

        for (idx, _mesh_type) in DefaultMeshType::iter().enumerate() {
            let id = ResourceTypeAndId {
                kind: lgn_graphics_data::runtime::Model::TYPE,
                id: ResourceId::from_str(DEFAULT_MESH_GUIDS[idx]).unwrap(),
            };
            model_meta_datas.insert(
                id,
                ModelMetaData {
                    meshes: vec![Mesh {
                        mesh_id: idx as u32,
                        material_id: u32::MAX,
                        material_index: u32::MAX,
                    }],
                },
            );
        }

        Self {
            model_meta_datas,
            default_model: ModelMetaData {
                meshes: vec![Mesh {
                    mesh_id: 1, // cube
                    material_id: u32::MAX,
                    material_index: u32::MAX,
                }],
            },
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(RenderStage::Prepare, on_model_events);
        app.add_system_to_stage(RenderStage::Prepare, debug_bounding_spheres);
    }

    pub fn add_model(&mut self, resource_id: ResourceTypeAndId, model: ModelMetaData) {
        self.model_meta_datas.insert(resource_id, model);
    }

    pub fn remove_model(&mut self, resource_id: ResourceTypeAndId) -> Option<ModelMetaData> {
        self.model_meta_datas.remove(&resource_id)
    }

    pub fn get_model_meta_data(
        &self,
        visual_component: &VisualComponent,
    ) -> (&ModelMetaData, bool) {
        if let Some(reference) = &visual_component.model_resource_id {
            if let Some(model_meta_data) = self.model_meta_datas.get(reference) {
                return (model_meta_data, true);
            }
            return (&self.default_model, false);
        }
        (&self.default_model, true)
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn on_model_events(
    renderer: Res<'_, Renderer>,
    mut model_manager: ResMut<'_, ModelManager>,
    mut mesh_manager: ResMut<'_, MeshManager>,
    material_manager: Res<'_, MaterialManager>,
    mut asset_loaded_events: EventReader<'_, '_, AssetRegistryEvent>,
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    for asset_loaded_event in asset_loaded_events.iter() {
        match asset_loaded_event {
            AssetRegistryEvent::AssetLoaded(mesh_reference)
                if mesh_reference.kind == ModelAsset::TYPE =>
            {
                if let Some(model_asset) = asset_registry
                    .get_untyped(*mesh_reference)
                    .and_then(|handle| handle.get::<ModelAsset>(&asset_registry))
                {
                    let meshes = model_asset
                        .meshes
                        .iter()
                        .map(|mesh| crate::components::Mesh {
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
                            indices: if !mesh.indices.is_empty() {
                                Some(mesh.indices.clone())
                            } else {
                                None
                            },
                            colors: if !mesh.colors.is_empty() {
                                Some(mesh.colors.iter().map(|v| Into::into(*v)).collect())
                            } else {
                                None
                            },
                            material_id: mesh.material.clone(),
                            bounding_sphere: crate::components::Mesh::calculate_bounding_sphere(
                                &mesh.positions,
                            ),
                        })
                        .collect::<Vec<_>>();

                    missing_visuals_tracker.add_visuals(*mesh_reference);
                    let ids = mesh_manager.add_meshes(&renderer, &meshes);

                    let meshes = meshes
                        .iter()
                        .zip(ids)
                        .map(|(mesh, mesh_id)| {
                            let material_id = mesh
                                .material_id
                                .as_ref()
                                .map(lgn_graphics_data::runtime::MaterialReferenceType::id);
                            Mesh {
                                mesh_id,
                                material_id: material_manager
                                    .gpu_data()
                                    .va_for_index(material_id, 0)
                                    as u32,
                                material_index: material_manager
                                    .gpu_data()
                                    .id_for_index(material_id, 0)
                                    as u32,
                            }
                        })
                        .collect();
                    model_manager.add_model(*mesh_reference, ModelMetaData { meshes });
                }
            }
            AssetRegistryEvent::AssetChanged(resource_id)
                if resource_id.kind == ModelAsset::TYPE =>
            {
                if let Some(_model_asset) = asset_registry
                    .get_untyped(*resource_id)
                    .and_then(|handle| handle.get::<ModelAsset>(&asset_registry))
                {
                    //model_manager.remove_model(*resource_id);
                    //mesh_manager.update_meshes(resource_id);
                }
            }
            AssetRegistryEvent::AssetUnloaded(resource_id)
                if resource_id.kind == ModelAsset::TYPE =>
            {
                model_manager.remove_model(*resource_id);
                //TODO: Remove the meshes
            }
            _ => (),
        }
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn debug_bounding_spheres(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    renderer_options: Res<'_, RendererOptions>,
    visuals: Query<'_, '_, (&VisualComponent, &Transform), Without<ManipulatorComponent>>,
) {
    if !renderer_options.show_bounding_spheres {
        return;
    }
    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for (visual, transform) in visuals.iter() {
                let (model_data, ready) = model_manager.get_model_meta_data(visual);
                if ready {
                    for mesh in &model_data.meshes {
                        let mesh_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                        //mesh_data.bounding_sphere
                        builder.add_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation + mesh_data.bounding_sphere.truncate(),
                                )
                                .with_scale(Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w)
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Sphere as u32,
                            Vec3::new(1.0, 1.0, 1.0),
                        );
                    }
                }
            }
        });
    });
}
