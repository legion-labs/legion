use lgn_ecs::prelude::Commands;
use lgn_graphics_renderer::{components::VisualComponent, resources::DefaultMeshType};
use lgn_math::prelude::Vec3;
use lgn_transform::prelude::{GlobalTransform, Transform, TransformBundle};

use crate::{runtime::PhysicsRigidSphere, RigidActorType};

pub(crate) fn spawn_random_sphere(mut commands: Commands<'_, '_>) {
    let translation = Vec3::new(0.0, 3.0, 0.7);
    commands
        .spawn()
        .insert_bundle(TransformBundle {
            local: Transform::from_translation(translation),
            global: GlobalTransform::from_translation(translation),
        })
        .insert(VisualComponent::new_default_mesh(
            DefaultMeshType::Sphere,
            (0xff, 0xff, 0x00).into(),
        ))
        .insert(PhysicsRigidSphere {
            actor_type: RigidActorType::Dynamic,
            radius: 0.25_f32,
        });
}
