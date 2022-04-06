use lgn_ecs::prelude::*;
use lgn_math::Vec4;

use crate::cgen::cgen_type::PickingData;

#[derive(Component)]
pub struct PickedComponent {
    picking_data: Vec<PickingData>,
}

impl PickedComponent {
    pub(crate) fn new() -> Self {
        Self {
            picking_data: Vec::new(),
        }
    }

    pub(crate) fn replace_picking_ids(
        &mut self,
        entity: Entity,
        picked_ids: &mut Vec<PickingData>,
        picked_entities: &mut Vec<Entity>,
    ) {
        self.picking_data.clear();

        let mut i = 0;
        while i < picked_ids.len() {
            if picked_entities[i] == entity {
                self.picking_data.push(picked_ids.swap_remove(i));
                picked_entities.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.picking_data.is_empty()
    }

    pub fn clear_picking_data(&mut self) {
        self.picking_data.clear();
    }

    pub fn get_closest_point(&self) -> Vec4 {
        let mut closest_point = Vec4::new(f32::MAX, f32::MAX, f32::MAX, f32::MAX);
        for picking_data in &self.picking_data {
            let picking_pos = Vec4::from(picking_data.picking_pos());
            if picking_pos.w < closest_point.w {
                closest_point = picking_pos;
            }
        }
        closest_point
    }
}
