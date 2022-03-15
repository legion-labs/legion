use std::{collections::BTreeMap, str::FromStr};

use lgn_app::App;
use lgn_core::BumpAllocatorPool;
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
use lgn_ecs::prelude::{Changed, Query, Res, ResMut, Without};
use lgn_math::Vec3;
use lgn_tracing::span_fn;
use lgn_transform::components::Transform;
use strum::IntoEnumIterator;

use crate::{
    components::{ManipulatorComponent, ModelComponent, VisualComponent},
    debug_display::DebugDisplay,
    labels::RenderStage,
    Renderer,
};

use super::{
    DefaultMeshType, GpuMaterialManager, MeshManager, MissingVisualTracker, RendererOptions,
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
        app.add_system_to_stage(RenderStage::Prepare, update_models);
    }

    pub fn add_model(&mut self, resource_id: ResourceTypeAndId, model: ModelMetaData) {
        self.model_meta_datas.insert(resource_id, model);
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
pub(crate) fn update_models(
    renderer: Res<'_, Renderer>,
    mut model_manager: ResMut<'_, ModelManager>,
    mut mesh_manager: ResMut<'_, MeshManager>,
    material_manager: Res<'_, MaterialManager>,
    updated_models: Query<'_, '_, &ModelComponent, Changed<ModelComponent>>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    for updated_model in updated_models.iter() {
        if let Some(mesh_reference) = &updated_model.model_id {
            missing_visuals_tracker.add_visuals(*mesh_reference);
            let ids = mesh_manager.add_meshes(&renderer, &updated_model.meshes);

            let mut meshes = Vec::new();
            // TODO: case when material hasn't been loaded
            for (idx, mesh) in updated_model.meshes.iter().enumerate() {
                meshes.push(Mesh {
                    mesh_id: ids[idx],
                    material_id: material_manager
                        .gpu_data()
                        .va_for_index(mesh.material_id.clone().map(|v| v.id()), 0)
                        as u32,
                    material_index: material_manager
                        .gpu_data()
                        .id_for_index(mesh.material_id.clone().map(|v| v.id()), 0)
                        as u32,
                });
            }
            model_manager.add_model(*mesh_reference, ModelMetaData { meshes });
        }
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn debug_bounding_spheres(
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
                            Transform::identity()
                                .with_translation(
                                    transform.translation + mesh_data.bounding_sphere.truncate(),
                                )
                                .with_scale(Vec3::new(4.0, 4.0, 4.0) * mesh_data.bounding_sphere.w)
                                .with_rotation(transform.rotation)
                                .compute_matrix(),
                            DefaultMeshType::Sphere as u32,
                            Vec3::new(1.0, 1.0, 1.0),
                        );
                    }
                }
            }
        });
    });
}
