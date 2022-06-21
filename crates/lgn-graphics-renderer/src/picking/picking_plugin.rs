use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_hierarchy::prelude::Parent;
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion},
};
use lgn_math::Vec2;
use lgn_tracing::span_fn;
use lgn_transform::{components::GlobalTransform, prelude::Transform};
use lgn_window::WindowResized;

use super::{picking_event::PickingEvent, ManipulatorManager, PickingIdContext, PickingManager};
use crate::{
    components::{
        CameraComponent, LightComponent, ManipulatorComponent, PickedComponent, RenderSurfaces,
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
        app.add_event::<PickingEvent>();

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
    mut query: Query<'_, '_, (Entity, &mut LightComponent), Added<LightComponent>>,
) {
    let mut picking_context = PickingIdContext::new(&picking_manager);

    for (entity, mut light) in query.iter_mut() {
        light.picking_id = picking_context.acquire_picking_id(entity);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn update_picking_components(
    picking_manager: Res<'_, PickingManager>,
    commands: Commands<'_, '_>,
    event_writer: EventWriter<'_, '_, PickingEvent>,
    query: Query<'_, '_, (Entity, &mut PickedComponent, Option<&ManipulatorComponent>)>,
    entities: Query<'_, '_, (Entity, &Transform)>,
    manipulator_entities: Query<'_, '_, (Entity, &ManipulatorComponent)>,
) {
    picking_manager.update_picking_components(
        commands,
        event_writer,
        query,
        manipulator_entities,
        entities,
    );
}

#[span_fn]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
fn update_picked_entity(
    picking_manager: Res<'_, PickingManager>,
    newly_picked_query: Query<
        '_,
        '_,
        (Entity, &Transform, &GlobalTransform),
        Added<PickedComponent>,
    >,
) {
    for (entity, local_transform, global_transform) in newly_picked_query.iter() {
        if Some(entity) == picking_manager.manipulated_entity() {
            picking_manager.set_base_picking_transforms(local_transform, global_transform);
        }
    }
}

#[span_fn]
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)]
fn update_manipulator_component(
    mut commands: Commands<'_, '_>,
    picking_manager: Res<'_, PickingManager>,
    manipulator_manager: Res<'_, ManipulatorManager>,
    mut event_writer: EventWriter<'_, '_, PickingEvent>,
    q_cameras: Query<
        '_,
        '_,
        (&CameraComponent, &GlobalTransform),
        (Without<PickedComponent>, Without<ManipulatorComponent>),
    >,
    render_surfaces: Res<'_, RenderSurfaces>,
    mut picked_query: Query<
        '_,
        '_,
        (
            Entity,
            &mut Transform,
            &GlobalTransform,
            &mut PickedComponent,
            Option<&Parent>,
        ),
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
    parent_query: Query<'_, '_, &GlobalTransform>,
) {
    let mut selected_part = usize::MAX;
    for (_entity, _transform, manipulator, picked_component) in manipulator_query.iter() {
        if picked_component.is_some() && picking_manager.mouse_button_down() {
            selected_part = manipulator.part_num;
        }
    }

    let mut update_manip_entity = false;
    let mut active_manipulator_part = false;
    for (entity, _transform, mut manipulator, picked_component) in manipulator_query.iter_mut() {
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
    for (entity, mut transform, global_transform, picked, parent) in picked_query.iter_mut() {
        if picking_manager.manipulated_entity() == Some(entity) {
            if active_manipulator_part {
                let (base_local_transform, base_global_transform) =
                    picking_manager.base_picking_transforms();

                let q_cameras = q_cameras
                    .iter()
                    .collect::<Vec<(&CameraComponent, &GlobalTransform)>>();
                if !q_cameras.is_empty() {
                    for render_surface in render_surfaces.iter() {
                        let mut screen_rect = picking_manager.screen_rect();
                        if screen_rect.x == 0.0 || screen_rect.y == 0.0 {
                            screen_rect = Vec2::new(
                                render_surface.extents().width() as f32,
                                render_surface.extents().height() as f32,
                            );
                        }

                        let parent_global_transform =
                            parent.map_or(GlobalTransform::identity(), |parent| {
                                *parent_query
                                    .get(parent.0)
                                    .unwrap_or(&GlobalTransform::identity())
                            });

                        *transform = manipulator_manager.manipulate_entity(
                            selected_part,
                            &base_local_transform,
                            &base_global_transform,
                            &parent_global_transform,
                            q_cameras[0].0,
                            q_cameras[0].1,
                            picking_manager.picked_pos(),
                            screen_rect,
                            picking_manager.current_cursor_pos(),
                        );
                    }
                }
            } else if update_manip_entity {
                if !picking_manager.mouse_button_down()
                    && (*transform) != picking_manager.base_picking_transforms().0
                {
                    event_writer.send(PickingEvent::ApplyTransaction(entity, *transform));
                }
                picking_manager.set_base_picking_transforms(&transform, global_transform);
                // Notify the transform
            }
            select_entity_transform = Some(*global_transform);
        } else if picked.is_empty() {
            commands.entity(entity).remove::<PickedComponent>();
        }
    }

    // Update the Manipulator Transform using the Manipulated Entity GlobalTransform
    for (_entity, mut transform, mut manipulator, _picked_component) in manipulator_query.iter_mut()
    {
        manipulator.active = false;
        if let Some(entity_transform) = select_entity_transform {
            if manipulator.part_type == manipulator_manager.current_manipulator_type() {
                manipulator_manager
                    .manipulator_transform_from_entity_transform(&entity_transform, &mut transform);
                manipulator.active = true;
            }
        }
    }
}
