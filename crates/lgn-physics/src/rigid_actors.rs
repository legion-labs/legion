use lgn_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut};
use lgn_tracing::prelude::error;
use lgn_transform::prelude::GlobalTransform;
use physx::{
    cooking::PxCooking,
    foundation::DefaultAllocator,
    prelude::{
        Geometry, GeometryType, Owner, Physics, PhysicsFoundation, PxTransform, RigidBody, Scene,
    },
};

use crate::{
    ConvertToCollisionGeometry, PxMaterial, PxScene, PxShape, RigidActorType, WithActorType,
};

pub(crate) fn create_rigid_actors<T>(
    query: Query<'_, '_, (Entity, &T, &GlobalTransform)>,
    mut physics: ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    cooking: Res<'_, Owner<PxCooking>>,
    mut scene: ResMut<'_, Owner<PxScene>>,
    mut default_material: ResMut<'_, Owner<PxMaterial>>,
    mut commands: Commands<'_, '_>,
) where
    T: Component + ConvertToCollisionGeometry + WithActorType,
{
    for (entity, physics_component, transform) in query.iter() {
        let mut entity_commands = commands.entity(entity);
        match physics_component.convert(&transform.scale, &mut physics, &cooking) {
            Ok(geometry) => {
                match physics_component.get_actor_type() {
                    RigidActorType::Dynamic => {
                        add_dynamic_actor_to_scene(
                            &mut physics,
                            &mut scene,
                            transform,
                            &geometry,
                            entity,
                            &mut default_material,
                        );
                    }
                    RigidActorType::Static => {
                        add_static_actor_to_scene(
                            &mut physics,
                            &mut scene,
                            transform,
                            &geometry,
                            entity,
                            &mut default_material,
                        );
                    }
                }

                entity_commands.insert(geometry);
            }
            Err(error) => {
                error!("failed to convert to collision geometry: {}", error);
            }
        }
        entity_commands.remove::<T>();
    }

    drop(query);
    drop(cooking);
}

pub(crate) fn add_dynamic_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) {
    debug_assert!(geometry.get_type() != GeometryType::Plane);
    let transform: PxTransform = transform.compute_matrix().into();
    let mut actor = physics
        .create_rigid_dynamic(
            transform,
            geometry,
            material,
            10_f32,
            PxTransform::default(),
            entity,
        )
        .expect("failed to create rigid dynamic actor");
    actor.set_angular_damping(0.5);
    scene.add_dynamic_actor(actor);
}

pub(crate) fn add_static_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) {
    let transform: PxTransform = transform.compute_matrix().into();
    let actor = physics
        .create_rigid_static(
            transform,
            geometry,
            material,
            PxTransform::default(),
            entity,
        )
        .expect("failed to create rigid static actor");
    scene.add_static_actor(actor);
}
