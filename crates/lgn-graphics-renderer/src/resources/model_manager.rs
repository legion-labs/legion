use std::{collections::BTreeMap, str::FromStr};

use lgn_app::App;
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_ecs::prelude::{Changed, Query, Res, ResMut};
use lgn_tracing::span_fn;
use strum::IntoEnumIterator;

use crate::{
    components::{ModelComponent, VisualComponent},
    labels::RenderStage,
    Renderer,
};

use super::{
    DefaultMeshType, GpuMaterialManager, MeshManager, MissingVisualTracker, DEFAULT_MESH_GUIDS,
};

pub struct Mesh {
    pub mesh_id: u32,
    pub material_id: u32,
    pub material_index: u32,
}

pub struct ModelMetaData {
    pub meshes: Vec<Mesh>,
}

impl ModelMetaData {
    pub(crate) fn has_material(&self) -> bool {
        for mesh in &self.meshes {
            if mesh.material_id != u32::MAX {
                return true;
            }
        }
        false
    }
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
                kind: ResourceType::from_raw(1),
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
    material_manager: Res<'_, GpuMaterialManager>,
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
                        .va_for_index(mesh.material_id.clone().map(|v| v.id()), 0)
                        as u32,
                    material_index: material_manager
                        .id_for_index(mesh.material_id.clone().map(|v| v.id()), 0)
                        as u32,
                });
            }
            model_manager.add_model(*mesh_reference, ModelMetaData { meshes });
        }
    }
}
