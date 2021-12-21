use lgn_ecs::prelude::*;

//#[derive(Component)]
pub struct PickedComponent {
    picking_ids: Vec<u32>,
}

impl PickedComponent {
    pub(crate) fn new() -> Self {
        Self {
            picking_ids: Vec::new(),
        }
    }

    pub(crate) fn replace_picking_ids(
        &mut self,
        entity: Entity,
        picked_ids: &mut Vec<u32>,
        picked_entities: &mut Vec<Entity>,
    ) {
        self.picking_ids.clear();

        let mut i = 0;
        while i < picked_ids.len() {
            if picked_entities[i] == entity {
                self.picking_ids.push(picked_ids.swap_remove(i));
                picked_entities.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.picking_ids.is_empty()
    }
}
