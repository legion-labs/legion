use lgn_graphics_data::Color;

use super::{Material, MaterialId, MaterialManager};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DefaultMaterialType {
    Default = 0,
    Gold,
    Silver,
    Bronze,
    BluePlastic,
    RoughMetal,
}

pub struct DefaultMaterials {
    default_material_ids: Vec<MaterialId>,
}

impl DefaultMaterials {
    pub fn new(material_manager: &mut MaterialManager) -> Self {
        let mut gold = Material::default();
        gold.base_color = Color::from((212, 175, 55));
        gold.metallic = 1.0;
        gold.roughness = 0.38;

        let mut silver = Material::default();
        silver.base_color = Color::from((168, 169, 173));
        silver.metallic = 1.0;
        silver.roughness = 0.38;

        let mut bronze = Material::default();
        bronze.base_color = Color::from((205, 127, 50));
        bronze.metallic = 1.0;
        bronze.roughness = 0.38;

        let mut blue_plastic = Material::default();
        blue_plastic.base_color = Color::from((20, 20, 150));
        blue_plastic.metallic = 0.0;
        blue_plastic.roughness = 0.15;

        let mut rough_metal = Material::default();
        rough_metal.base_color = Color::from((127, 64, 25));
        rough_metal.metallic = 1.0;
        rough_metal.specular = 0.2;
        rough_metal.roughness = 0.5;

        let default_material_ids = vec![
            material_manager.new_material(None).material_id,
            material_manager.new_material(Some(gold)).material_id,
            material_manager.new_material(Some(silver)).material_id,
            material_manager.new_material(Some(bronze)).material_id,
            material_manager
                .new_material(Some(blue_plastic))
                .material_id,
            material_manager.new_material(Some(rough_metal)).material_id,
        ];

        Self {
            default_material_ids,
        }
    }

    pub fn get_material_id(&self, material_type: DefaultMaterialType) -> MaterialId {
        self.default_material_ids[material_type as usize]
    }
}
