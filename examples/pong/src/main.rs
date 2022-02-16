//! Specialized runtime server, with additional services, that will run the pong demo.
//! Once scripting is properly supported, the services will be in data and will use the
//! standard runtime server.

// crate-specific lint exceptions:
//#![allow()]

use dolly::{prelude::*, rig::CameraRig};
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_math::prelude::*;
use lgn_renderer::components::CameraComponent;
use runtime_srv::{build_runtime, start_runtime};

fn main() {
    let mut app = build_runtime(
        None,
        "examples/pong/data",
        "(1d9ddd99aad89045,b3440a7c-ba07-5628-e7f8-bb89ed5de900)",
    );

    app.add_startup_system_to_stage(StartupStage::PostStartup, game_setup);

    start_runtime(&mut app);
}

fn game_setup(mut cameras: Query<'_, '_, &mut CameraComponent>) {
    for mut camera in cameras.iter_mut() {
        let eye = Vec3::new(0.0, 0.0, 7.0);

        camera.camera_rig = CameraRig::builder()
            .with(Position::new(eye))
            .with(YawPitch::new())
            .build();

        camera.speed = 5_f32;
        camera.rotation_speed = 30_f32;
    }
}
