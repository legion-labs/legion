//! Specialized runtime server, with additional services, that will run the pong demo.
//! Once scripting is properly supported, the services will be in data and will use the
//! standard runtime server.

// crate-specific lint exceptions:
//#![allow()]

use dolly::{prelude::*, rig::CameraRig};
use lgn_app::prelude::*;
use lgn_core::Name;
use lgn_ecs::prelude::*;
use lgn_input::mouse::MouseMotion;
use lgn_math::Vec3;
use lgn_renderer::components::CameraComponent;
use lgn_transform::components::Transform;
use runtime_srv::{build_runtime, start_runtime};

fn main() {
    let mut app = build_runtime(
        None,
        "examples/pong/data",
        "", // using game.manifest
    );

    app.insert_resource(GameState::default())
        .add_startup_system_to_stage(StartupStage::PostStartup, game_setup)
        .add_system(update_mouse)
        .add_system(game_logic);

    start_runtime(&mut app);
}

#[derive(Default)]
struct GameState {
    left_paddle_id: Option<u32>,
    right_paddle_id: Option<u32>,
    ball_id: Option<u32>,
    velocity: f32,
    direction: Vec3,
    paddle_delta: f32,
}

fn game_setup(mut cameras: Query<'_, '_, &mut CameraComponent>, mut state: ResMut<'_, GameState>) {
    for mut camera in cameras.iter_mut() {
        let eye = Vec3::new(0.0, 0.0, 7.0);

        camera.camera_rig = CameraRig::builder()
            .with(Position::new(eye))
            .with(YawPitch::new())
            .build();

        camera.speed = 5_f32;
        camera.rotation_speed = 30_f32;
    }

    state.velocity = 0.7;

    state.direction.x = rand::random::<f32>() - 0.5_f32;
    state.direction.y = rand::random::<f32>() - 0.5_f32;
    state.direction = state.direction.normalize() * 0.1_f32;
}

fn update_mouse(
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut state: ResMut<'_, GameState>,
) {
    // aggregate mouse movement
    let mut mouse_delta_x = 0_f32;
    for motion_event in mouse_motion_events.iter() {
        mouse_delta_x += motion_event.delta.x;
    }

    state.paddle_delta = mouse_delta_x / 200_f32;
}

fn game_logic(
    mut entities: Query<'_, '_, (Entity, &Name, &mut Transform)>,
    mut state: ResMut<'_, GameState>,
) {
    if state.ball_id.is_none() || state.left_paddle_id.is_none() || state.right_paddle_id.is_none()
    {
        lookup_entities_by_name(&entities, &mut state);
        if state.ball_id.is_none()
            || state.left_paddle_id.is_none()
            || state.right_paddle_id.is_none()
        {
            return;
        }
    }
    let ball_id = state.ball_id.unwrap();
    let left_paddle_id = state.left_paddle_id.unwrap();
    let right_paddle_id = state.right_paddle_id.unwrap();

    // update paddles
    let mut left_paddle = 0.0;
    let mut right_paddle = 0.0;
    for (entity, _name, mut transform) in entities.iter_mut() {
        if entity.id() == left_paddle_id {
            transform.translation.y -= state.paddle_delta;
            transform.translation.y = transform.translation.y.clamp(-2.0, 2.0);
            left_paddle = transform.translation.y;
        } else if entity.id() == right_paddle_id {
            transform.translation.y += state.paddle_delta;
            transform.translation.y = transform.translation.y.clamp(-2.0, 2.0);
            right_paddle = transform.translation.y;
        }
    }

    // update ball
    for (entity, _name, mut transform) in entities.iter_mut() {
        if entity.id() == ball_id {
            // Ball
            let mut position = transform.translation;
            if position.x < -3.0 || position.x > 3.0 {
                state.direction.x = -state.direction.x;
            }
            if position.y < -2.0 || position.y > 2.0 {
                state.direction.y = -state.direction.y;
            }

            position.x = position.x.clamp(-3.0, 3.0);
            position.y = position.y.clamp(-2.0, 2.0);

            // check for collision with paddles (dimensions = 0.2 x 1.0 x 0.2)
            // Note: x-axis is inverted so values decrease towards the right
            let new_position = position + state.velocity * state.direction;
            if state.direction.x > 0.0 {
                // moving left
                if (position.x < 2.3
                    && new_position.x >= 2.3
                    && position.y > left_paddle - 0.5
                    && position.y < left_paddle + 0.5)
                    || (position.x < -2.5
                        && new_position.x >= -2.5
                        && position.y > right_paddle - 0.5
                        && position.y < right_paddle + 0.5)
                {
                    state.direction.x = -state.direction.x;
                }
            } else {
                // moving right
                if (position.x > -2.3
                    && new_position.x <= -2.3
                    && position.y > right_paddle - 0.5
                    && position.y < right_paddle + 0.5)
                    || (position.x > 2.5
                        && new_position.x <= 2.5
                        && position.y > left_paddle - 0.5
                        && position.y < left_paddle + 0.5)
                {
                    state.direction.x = -state.direction.x;
                }
            }

            if state.direction.y > 0.0 {
                // moving up
                let left_bottom = left_paddle - 0.5;
                let right_bottom = right_paddle - 0.5;
                if (position.y < left_bottom
                    && new_position.y >= left_bottom
                    && position.x > 2.3
                    && position.x < 2.5)
                    || (position.y < right_bottom
                        && new_position.y >= right_bottom
                        && position.x < -2.3
                        && position.x > -2.5)
                {
                    state.direction.y = -state.direction.y;
                }
            } else {
                // moving down
                let left_top = left_paddle + 0.5;
                let right_top = right_paddle + 0.5;
                if (position.y > left_top
                    && new_position.y <= left_top
                    && position.x > 2.3
                    && position.x < 2.5)
                    || (position.y > right_top
                        && new_position.y <= right_top
                        && position.x < -2.3
                        && position.x > -2.5)
                {
                    state.direction.y = -state.direction.y;
                }
            }

            transform.translation = position;
            transform.translation += state.velocity * state.direction;
        }
    }
}

fn lookup_entities_by_name(
    entities: &Query<'_, '_, (Entity, &Name, &mut Transform)>,
    state: &mut ResMut<'_, GameState>,
) {
    if state.ball_id.is_none() {
        state.ball_id = lookup_entity_by_name("Ball", entities);
    }

    if state.left_paddle_id.is_none() {
        state.left_paddle_id = lookup_entity_by_name("Pad Left", entities);
    }

    if state.right_paddle_id.is_none() {
        state.right_paddle_id = lookup_entity_by_name("Pad Right", entities);
    }
}

fn lookup_entity_by_name(
    name: &'static str,
    entities: &Query<'_, '_, (Entity, &Name, &mut Transform)>,
) -> Option<u32> {
    entities
        .iter()
        .find(|(_entity, entity_name, _transform)| entity_name.as_str() == name)
        .map(|(entity, _entity_name, _transform)| entity.id())
}
