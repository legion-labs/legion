use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_input::mouse::{MouseButtonInput, MouseMotion};
use lgn_math::Vec2;
use lgn_transform::prelude::Transform;
use lgn_window::WindowResized;

use super::{ManipulatorManager, PickingManager, PositionComponents};
use crate::components::{
    CameraComponent, ManipulatorComponent, PickedComponent, RenderSurface, StaticMesh,
};

pub struct PickingPlugin {
    has_window: bool,
}

impl PickingPlugin {
    pub fn new(has_window: bool) -> Self {
        Self { has_window }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum PickingSystemLabel {
    PickedComponent,
    PickedEntity,
    Manipulator,
}

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        let picking_manager = PickingManager::new(4096);
        app.insert_resource(picking_manager);

        app.add_system_to_stage(CoreStage::PreUpdate, gather_input);
        if self.has_window {
            app.add_system_to_stage(CoreStage::PreUpdate, gather_window_resize);
        }
        app.add_system_to_stage(CoreStage::PreUpdate, static_meshes_added);

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_picking_components
                .before(PickingSystemLabel::PickedEntity)
                .label(PickingSystemLabel::PickedComponent),
        );

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_picked_entity
                .before(PickingSystemLabel::Manipulator)
                .label(PickingSystemLabel::PickedEntity),
        );

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_manipulator_component.label(PickingSystemLabel::Manipulator),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn gather_input(
    picking_manager: Res<'_, PickingManager>,
    mut cursor_button: EventReader<'_, '_, MouseButtonInput>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
) {
    for cursor_button_event in cursor_button.iter() {
        picking_manager.set_mouse_button_input(cursor_button_event);
    }

    for motion_event in mouse_motion_events.iter() {
        picking_manager.set_mouse_moition_event(motion_event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn gather_window_resize(
    picking_manager: Res<'_, PickingManager>,
    mut window_resized_events: EventReader<'_, '_, WindowResized>,
) {
    for window_resized_event in window_resized_events.iter() {
        picking_manager.set_screen_rect(&Vec2::new(
            window_resized_event.width,
            window_resized_event.height,
        ));
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn static_meshes_added(
    picking_manager: Res<'_, PickingManager>,
    mut query: Query<
        '_,
        '_,
        (Entity, &mut StaticMesh),
        (Added<StaticMesh>, Without<ManipulatorComponent>),
    >,
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
    query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            &mut PickedComponent,
            Option<&ManipulatorComponent>,
        ),
    >,
    manipulator_entities: Query<'_, '_, (Entity, &ManipulatorComponent)>,
) {
    picking_manager.update_picking_components(commands, query, manipulator_entities);
}

#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn update_picked_entity(
    picking_manager: Res<'_, PickingManager>,
    mut newly_picked_query: Query<
        '_,
        '_,
        (
            Entity,
            &mut Transform,
            &PickedComponent,
            Option<&ManipulatorComponent>,
        ),
        Added<PickedComponent>,
    >,
) {
    for (entity, transform, picked, manipulator) in newly_picked_query.iter_mut() {
        if manipulator.is_some() {
            picking_manager.set_picking_start_pos(picked.get_closest_point());
        } else {
            picking_manager.set_manip_entity(entity, &transform);
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn update_manipulator_component(
    mut commands: Commands<'_, '_>,
    picking_manager: Res<'_, PickingManager>,
    manipulator_manager: Res<'_, ManipulatorManager>,
    q_cameras: Query<
        '_,
        '_,
        &CameraComponent,
        (Without<PickedComponent>, Without<ManipulatorComponent>),
    >,
    q_render_surfaces: Query<'_, '_, &RenderSurface>,
    mut picked_query: Query<
        '_,
        '_,
        (Entity, &mut Transform, &mut PickedComponent),
        Without<ManipulatorComponent>,
    >,
    mut manipulator_query: Query<
        '_,
        '_,
        (
            Entity,
            &mut Transform,
            &mut ManipulatorComponent,
            Option<&mut PickedComponent>,
        ),
    >,
) {
    let mut selected_part = usize::MAX;
    for (_entity, _transform, manipulator, picked_component) in manipulator_query.iter() {
        if picked_component.is_some() && picking_manager.mouse_button_down() {
            selected_part = manipulator.part_num;
        }
    }

    let mut update_manip_entity = false;
    let mut active_manipulator_part = PositionComponents::None;
    for (entity, _transform, mut manipulator, picked_component) in manipulator_query.iter_mut() {
        manipulator.selected = false;
        if selected_part != usize::MAX
            && PositionComponents::from_component_id(selected_part)
                == PositionComponents::from_component_id(manipulator.part_num)
        {
            active_manipulator_part = PositionComponents::from_component_id(selected_part);
            manipulator.selected = true;
        } else if picked_component.is_some() {
            commands.entity(entity).remove::<PickedComponent>();
            update_manip_entity = true;
        }
    }

    let mut select_entity_transform = None;
    for (entity, mut transform, picked) in picked_query.iter_mut() {
        if entity == picking_manager.manipulated_entity() {
            if active_manipulator_part != PositionComponents::None {
                let (base_transform, picking_pos) = picking_manager.base_picking_data();

                let q_cameras = q_cameras.iter().collect::<Vec<&CameraComponent>>();
                if !q_cameras.is_empty() {
                    for render_surface in q_render_surfaces.iter() {
                        let mut screen_rect = picking_manager.screen_rect();
                        if screen_rect.x == 0.0 || screen_rect.y == 0.0 {
                            screen_rect = Vec2::new(
                                render_surface.extents().width() as f32,
                                render_surface.extents().height() as f32,
                            );
                        }

                        *transform = manipulator_manager.manipulate_entity(
                            active_manipulator_part,
                            &base_transform,
                            q_cameras[0],
                            picking_pos,
                            screen_rect,
                            picking_manager.current_cursor_pos(),
                        );
                    }
                }
            } else if update_manip_entity {
                picking_manager.set_manip_entity(entity, &transform);
            }
            select_entity_transform = Some(*transform);
        } else if picked.is_empty() {
            commands.entity(entity).remove::<PickedComponent>();
        }
    }

    for (_entity, mut transform, mut manipulator, _picked_component) in manipulator_query.iter_mut()
    {
        manipulator.active = false;

        if let Some(entity_transform) = select_entity_transform {
            if manipulator.part_type == manipulator_manager.curremt_manipulator_type() {
                transform.translation =
                    entity_transform.translation + manipulator.local_translation;
                manipulator.active = true;
            }
        }
    }
}
