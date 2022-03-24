use lgn_app::prelude::{App, CoreStage};
use lgn_core::{prelude::Timer, FixedTimestep, Time};
use lgn_ecs::prelude::{Commands, Component, Entity, Query, Res, SystemStage};
use lgn_graphics_renderer::{components::VisualComponent, resources::DefaultMeshType};
use lgn_math::prelude::Vec3;
use lgn_tracing::{info, warn};
use lgn_transform::prelude::{GlobalTransform, Transform, TransformBundle};

use crate::{runtime::PhysicsRigidSphere, RigidActorType};

pub(crate) fn build(app: &mut App) {
    app.add_stage_after(
        CoreStage::PreUpdate,
        "random_spawning",
        SystemStage::parallel()
            .with_run_criteria(FixedTimestep::step(1.0))
            .with_system(spawn_random_sphere),
    );

    app.add_system(tick);
}

fn spawn_random_sphere(mut commands: Commands<'_, '_>) {
    let translation = Vec3::new(0.0, 3.0, 0.7);
    let entity = commands
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
        })
        .insert(Timebomb {
            timer: Timer::from_seconds(5.0, false),
        })
        .id();
    info!("spawning entity {}", entity.id());
}

#[derive(Component)]
struct Timebomb {
    timer: Timer,
}

fn tick(
    mut commands: Commands<'_, '_>,
    mut query: Query<'_, '_, (Entity, &mut Timebomb)>,
    time: Res<'_, Time>,
) {
    for (entity, mut timebomb) in query.iter_mut() {
        timebomb.timer.tick(time.delta());
        if timebomb.timer.finished() {
            commands.entity(entity).despawn();
            warn!("despawning entity {}", entity.id());
        }
    }

    drop(time);
}
