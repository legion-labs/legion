use lgn_ecs::prelude::*;
use physx::prelude::*;

use crate::PxRigidDynamic;

#[derive(Component)]
pub struct RigidDynamicActor {
    pub(crate) actor: Owner<PxRigidDynamic>,
}

// fn create_dynamic() {
//     let mut sphere_actor = physics
//         .create_rigid_dynamic(
//             PxTransform::from_translation(&PxVec3::new(0.0, 40.0, 100.0)),
//             &sphere_geo,
//             material.as_mut(),
//             10.0,
//             PxTransform::default(),
//             (),
//         )
//         .unwrap();
//     sphere_actor.set_angular_damping(0.5);
//     sphere_actor.set_rigid_body_flag(RigidBodyFlag::EnablePoseIntegrationPreview, true);
//     scene.add_dynamic_actor(sphere_actor);
// }
