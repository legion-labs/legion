use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion},
};
use lgn_math::Vec2;
use lgn_tracing::span_fn;
use lgn_transform::prelude::{Parent, Transform};
use lgn_window::WindowResized;
use std::ops::Deref;

use super::{ManipulatorManager, PickingManager};
use crate::{
    components::{
        CameraComponent, LightComponent, ManipulatorComponent, PickedComponent, RenderSurface,
    },
    CommandBufferLabel, RenderStage,
};

pub struct PickingPlugin {}

impl PickingPlugin {}

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

        app.add_system_to_stage(CoreStage::PostUpdate, gather_input);
        app.add_system_to_stage(CoreStage::PostUpdate, gather_window_resize);

        app.add_system_to_stage(CoreStage::PostUpdate, lights_added);

        app.add_system_to_stage(
            RenderStage::Render,
            update_picking_components
                .after(CommandBufferLabel::Generate)
                .label(PickingSystemLabel::PickedComponent),
        );

        app.add_system_to_stage(
            RenderStage::Render,
            update_picked_entity
                .after(PickingSystemLabel::PickedComponent)
                .label(PickingSystemLabel::PickedEntity),
        );

        app.add_system_to_stage(
            RenderStage::Render,
            update_manipulator_component
                .after(PickingSystemLabel::PickedEntity)
                .label(PickingSystemLabel::Manipulator),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn gather_input(
    picking_manager: Res<'_, PickingManager>,
    manipulator_manager: Res<'_, ManipulatorManager>,
    mut cursor_button: EventReader<'_, '_, MouseButtonInput>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    for cursor_button_event in cursor_button.iter() {
        picking_manager.set_mouse_button_input(cursor_button_event);
    }

    for motion_event in mouse_motion_events.iter() {
        picking_manager.set_mouse_motion_event(motion_event);
    }

    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            manipulator_manager.change_manipulator(key_code);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn gather_window_resize(
    picking_manager: Res<'_, PickingManager>,
    mut window_resized_events: EventReader<'_, '_, WindowResized>,
) {
    for window_resized_event in window_resized_events.iter() {
        picking_manager.set_screen_rect(Vec2::new(
            window_resized_event.width,
            window_resized_event.height,
        ));
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn lights_added(
    picking_manager: Res<'_, PickingManager>,
    mut query: Query<
        '_,
        '_,
        (Entity, &mut LightComponent),
        (Added<LightComponent>, Without<ManipulatorComponent>),
    >,
) {
    let mut picking_block = picking_manager.acquire_picking_id_block();

    for (entity, mut light) in query.iter_mut() {
        light.picking_id = picking_block.acquire_picking_id(entity).unwrap();
    }

    picking_manager.release_picking_id_block(picking_block);
}

#[span_fn]
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

#[span_fn]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn update_picked_entity(
    picking_manager: Res<'_, PickingManager>,
    mut newly_picked_query: Query<
        '_,
        '_,
        (Entity, Option<&Parent>, &mut Transform),
        (Added<PickedComponent>, Without<ManipulatorComponent>),
    >,
) {
    for (entity, parent, transform) in newly_picked_query.iter_mut() {
        picking_manager.set_manip_entity(entity, parent.map(|p| *p.deref()), &transform);
    }
}

#[span_fn]
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
        (
            Entity,
            Option<&Parent>,
            &mut Transform,
            &mut PickedComponent,
        ),
        Without<ManipulatorComponent>,
    >,
    mut manipulator_query: Query<
        '_,
        '_,
        (
            Entity,
            Option<&Parent>,
            &mut Transform,
            &mut ManipulatorComponent,
            Option<&mut PickedComponent>,
        ),
    >,
) {
    let mut selected_part = usize::MAX;
    for (_entity, _parent, _transform, manipulator, picked_component) in manipulator_query.iter() {
        if picked_component.is_some() && picking_manager.mouse_button_down() {
            selected_part = manipulator.part_num;
        }
    }

    let mut update_manip_entity = false;
    let mut active_manipulator_part = false;
    for (entity, _parent, _transform, mut manipulator, picked_component) in
        manipulator_query.iter_mut()
    {
        manipulator.selected = false;
        if selected_part != usize::MAX
            && manipulator_manager.match_manipulator_parts(
                selected_part,
                manipulator.part_type,
                manipulator.part_num,
            )
        {
            active_manipulator_part = true;
            manipulator.selected = true;
        } else if picked_component.is_some() {
            commands.entity(entity).remove::<PickedComponent>();
            update_manip_entity = true;
        }
    }

    let mut select_entity_transform = None;
    for (entity, parent, mut transform, picked) in picked_query.iter_mut() {
        if entity == picking_manager.manipulated_entity() {
            if active_manipulator_part {
                let base_transform = picking_manager.base_picking_transform();

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
                            selected_part,
                            &base_transform,
                            q_cameras[0],
                            picking_manager.picked_pos(),
                            screen_rect,
                            picking_manager.current_cursor_pos(),
                        );
                    }
                }
            } else if update_manip_entity {
                picking_manager.set_manip_entity(entity, parent.map(|p| *p.deref()), &transform);
            }
            select_entity_transform = Some(*transform);
        } else if picked.is_empty() {
            commands.entity(entity).remove::<PickedComponent>();
        }
    }

    for (entity, parent, mut transform, mut manipulator, _picked_component) in
        manipulator_query.iter_mut()
    {
        manipulator.active = false;

        if let Some(entity_transform) = select_entity_transform {
            if manipulator.part_type == manipulator_manager.current_manipulator_type() {
                manipulator_manager
                    .manipulator_transform_from_entity_transform(&entity_transform, &mut transform);

                if parent.map(Parent::deref) != picking_manager.manipulated_entity_parent().as_ref()
                {
                    if let Some(parent) = picking_manager.manipulated_entity_parent() {
                        commands.entity(entity).insert(Parent(parent));
                    } else {
                        commands.entity(entity).remove::<Parent>();
                    }
                }
                manipulator.active = true;
            }
        }
    }
}
