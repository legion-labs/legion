use std::ops::Deref;

use crossbeam_channel::{Receiver, Sender};
use lgn_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut, Without};
use lgn_tracing::prelude::error;
use lgn_transform::prelude::GlobalTransform;
use physx::{
    cooking::PxCooking,
    foundation::DefaultAllocator,
    prelude::{
        Geometry, GeometryType, Owner, Physics, PhysicsFoundation, PxTransform, RigidBody, Scene,
    },
    traits::Class,
};
use physx_sys::PxActor;

use crate::{
    collision_geometry::{CollisionGeometry, ConvertToCollisionGeometry},
    PxMaterial, PxScene, PxShape, RigidActorType, WithActorType,
};

#[derive(Component)]
pub(crate) struct RigidActor {
    actor: ActorMutPtr,
    event_sender: Sender<ActorDestructionEvent>,
}

pub(crate) struct ActorDestructionEvent(ActorMutPtr);

impl From<&RigidActor> for ActorDestructionEvent {
    fn from(rigid_actor: &RigidActor) -> Self {
        Self(rigid_actor.actor)
    }
}

impl Deref for ActorDestructionEvent {
    type Target = ActorMutPtr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ActorMutPtr(*mut PxActor);

impl Deref for ActorMutPtr {
    type Target = *mut PxActor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// SAFETY: the actors are kept alive by the physics scene, the pointer is only used
#[allow(unsafe_code)]
unsafe impl Send for ActorMutPtr {}
#[allow(unsafe_code)]
unsafe impl Sync for ActorMutPtr {}

pub(crate) fn create_rigid_actors<T>(
    query: Query<'_, '_, (Entity, &T, &GlobalTransform), Without<CollisionGeometry>>,
    mut physics: ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    cooking: Res<'_, Owner<PxCooking>>,
    mut scene: ResMut<'_, Owner<PxScene>>,
    mut default_material: ResMut<'_, Owner<PxMaterial>>,
    mut commands: Commands<'_, '_>,
    sender: Res<'_, Sender<ActorDestructionEvent>>,
) where
    T: Component + ConvertToCollisionGeometry + WithActorType,
{
    for (entity, physics_component, transform) in query.iter() {
        let mut entity_commands = commands.entity(entity);
        match physics_component.convert(&transform.scale, &mut physics, &cooking) {
            Ok(geometry) => {
                let actor = ActorMutPtr(match physics_component.get_actor_type() {
                    RigidActorType::Dynamic => add_dynamic_actor_to_scene(
                        &mut physics,
                        &mut scene,
                        transform,
                        &geometry,
                        entity,
                        &mut default_material,
                    ),
                    RigidActorType::Static => add_static_actor_to_scene(
                        &mut physics,
                        &mut scene,
                        transform,
                        &geometry,
                        entity,
                        &mut default_material,
                    ),
                });

                entity_commands.insert(geometry);
                entity_commands.insert(RigidActor {
                    actor,
                    event_sender: sender.clone(),
                });
            }
            Err(error) => {
                error!("failed to convert to collision geometry: {}", error);
            }
        }
    }

    drop(query);
    drop(cooking);
    drop(sender);
}

pub(crate) fn add_dynamic_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) -> *mut PxActor {
    debug_assert!(geometry.get_type() != GeometryType::Plane); // plane can only be used for static actors
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
    let actor_ptr: *mut PxActor = actor.as_mut_ptr();
    scene.add_dynamic_actor(actor);
    actor_ptr
}

pub(crate) fn add_static_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) -> *mut PxActor {
    let transform: PxTransform = transform.compute_matrix().into();
    let mut actor = physics
        .create_rigid_static(
            transform,
            geometry,
            material,
            PxTransform::default(),
            entity,
        )
        .expect("failed to create rigid static actor");
    let actor_ptr: *mut PxActor = actor.as_mut_ptr();
    scene.add_static_actor(actor);
    actor_ptr
}

pub(crate) fn cleanup_rigid_actors(
    mut scene: ResMut<'_, Owner<PxScene>>,
    receiver: Res<'_, Receiver<ActorDestructionEvent>>,
) {
    let wake_touching = true;
    for event in receiver.try_iter() {
        #[allow(unsafe_code)]
        unsafe {
            physx_sys::PxScene_removeActor_mut(scene.as_mut_ptr(), **event, wake_touching);
        }
    }

    drop(receiver);
}

impl Drop for RigidActor {
    fn drop(&mut self) {
        let _result = self.event_sender.send((&*self).into());
    }
}
