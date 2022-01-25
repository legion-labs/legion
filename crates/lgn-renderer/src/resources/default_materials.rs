use lgn_ecs::prelude::{Commands, Entity};
use lgn_graphics_data::Color;

use crate::components::MaterialComponent;

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
    default_material_ids: Vec<Entity>,
}

impl DefaultMaterials {
    pub fn new() -> Self {
        Self {
            default_material_ids: Vec::new(),
        }
    }

    pub fn initialize(&mut self, mut commands: Commands<'_, '_>) {
        let default = MaterialComponent::default();

        let mut gold = MaterialComponent::default();
        gold.base_color = Color::from((212, 175, 55));
        gold.metallic = 1.0;
        gold.roughness = 0.38;

        let mut silver = MaterialComponent::default();
        silver.base_color = Color::from((168, 169, 173));
        silver.metallic = 1.0;
        silver.roughness = 0.38;

        let mut bronze = MaterialComponent::default();
        bronze.base_color = Color::from((205, 127, 50));
        bronze.metallic = 1.0;
        bronze.roughness = 0.38;

        let mut blue_plastic = MaterialComponent::default();
        blue_plastic.base_color = Color::from((20, 20, 150));
        blue_plastic.metallic = 0.0;
        blue_plastic.roughness = 0.15;

        let mut rough_metal = MaterialComponent::default();
        rough_metal.base_color = Color::from((127, 64, 25));
        rough_metal.metallic = 1.0;
        rough_metal.specular = 0.2;
        rough_metal.roughness = 0.5;

        self.default_material_ids = vec![
            commands.spawn().insert(default).id(),
            commands.spawn().insert(gold).id(),
            commands.spawn().insert(silver).id(),
            commands.spawn().insert(bronze).id(),
            commands.spawn().insert(blue_plastic).id(),
            commands.spawn().insert(rough_metal).id(),
        ];
    }

    pub fn get_material_id(&self, material_type: DefaultMaterialType) -> Entity {
        self.default_material_ids[material_type as usize]
    }
}
