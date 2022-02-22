use std::collections::BTreeMap;

use lgn_data_runtime::{ResourceId, ResourceTypeAndId};

use crate::components::VisualComponent;

pub struct SubMesh {
    pub mesh_id: u32,
    pub material_id: u32,
}

pub struct ModelMetaData {
    pub meshes: Vec<SubMesh>,
}

pub struct ModelManager {
    model_meta_datas: BTreeMap<ResourceTypeAndId, ModelMetaData>,
    default_model: ModelMetaData,
}

impl ModelManager {
    pub fn new() -> Self {
        ModelManager {
            model_meta_datas: BTreeMap::new(),
            default_model: ModelMetaData {
                meshes: vec![SubMesh {
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
        if let Some(reference) = &visual_component.mesh_reference {
            if let Some(model_meta_data) = self.model_meta_datas.get(&reference) {
                return (model_meta_data, true);
            }
            return (&self.default_model, false);
        }
        (&self.default_model, true)
    }
}
