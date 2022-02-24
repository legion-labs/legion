use std::{collections::BTreeMap, str::FromStr};

use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use strum::IntoEnumIterator;

use crate::components::VisualComponent;

use super::{DefaultMeshType, DEFAULT_MESH_GUIDS};

pub struct Mesh {
    pub mesh_id: u32,
    pub material_id: u32,
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
                kind: ResourceType::from_raw(1),
                id: ResourceId::from_str(DEFAULT_MESH_GUIDS[idx]).unwrap(),
            };
            model_meta_datas.insert(
                id,
                ModelMetaData {
                    meshes: vec![Mesh {
                        mesh_id: idx as u32,
                        material_id: u32::MAX,
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
                }],
            },
        }
    }

    pub fn add_model(&mut self, resource_id: ResourceTypeAndId, model: ModelMetaData) {
        self.model_meta_datas.insert(resource_id, model);
    }

    pub fn get_model_meta_data(
        &self,
        visual_component: &VisualComponent,
    ) -> (&ModelMetaData, bool) {
        if let Some(reference) = &visual_component.model_reference {
            if let Some(model_meta_data) = self.model_meta_datas.get(&reference) {
                return (model_meta_data, true);
            }
            return (&self.default_model, false);
        }
        (&self.default_model, true)
    }
}
