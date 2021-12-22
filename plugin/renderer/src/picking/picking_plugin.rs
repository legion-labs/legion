use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_input::mouse::MouseButtonInput;
use lgn_math::Vec2;
use lgn_window::WindowResized;

use super::PickingManager;
use crate::components::{PickedComponent, StaticMesh};

pub struct PickingPlugin {
    has_window: bool,
}

impl PickingPlugin {
    pub fn new(has_window: bool) -> Self {
        Self { has_window }
    }
}

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        let picking_manager = PickingManager::new(4096);
        app.insert_resource(picking_manager);

        if self.has_window {
            app.add_system_to_stage(CoreStage::PreUpdate, gather_input_window);
            app.add_system_to_stage(CoreStage::PreUpdate, static_meshes_added);

            app.add_system_to_stage(CoreStage::PostUpdate, update_picking_components);
        }
    }
}

fn gather_input_window(
    mut picking_manager: ResMut<'_, PickingManager>,
    mut cursor_button: EventReader<'_, '_, MouseButtonInput>,
    mut window_resized_events: EventReader<'_, '_, WindowResized>,
) {
    for cursor_button_event in cursor_button.iter() {
        picking_manager.set_mouse_button_input(cursor_button_event);
    }

    for window_resized_event in window_resized_events.iter() {
        picking_manager.set_screen_rect(&Vec2::new(
            window_resized_event.width,
            window_resized_event.height,
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn static_meshes_added(
    picking_manager: Res<'_, PickingManager>,
    mut query: Query<'_, '_, (Entity, &mut StaticMesh), Added<StaticMesh>>,
) {
    let mut picking_block = picking_manager.aquire_picking_id_block();

    for (entity, mut mesh) in query.iter_mut() {
        mesh.picking_id = picking_block.aquire_picking_id(entity).unwrap();
    }

    picking_manager.release_picking_id_block(picking_block);
}

#[allow(clippy::needless_pass_by_value)]
fn update_picking_components(
    picking_manager: Res<'_, PickingManager>,
    commands: Commands<'_, '_>,
    query: Query<'_, '_, (Entity, &mut PickedComponent)>,
) {
    picking_manager.update_picking_components(commands, query);
}
