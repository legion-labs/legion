use lgn_ecs::prelude::*;
use lgn_transform::prelude::*;
use physx::{foundation::DefaultAllocator, prelude::*};

use crate::{
    runtime::{CollisionGeometry, PhysicsRigidActor},
    PxMaterial, PxScene, PxShape, RigidActorType,
};

#[derive(Component)]
pub(crate) struct RigidDynamicActor {
    geometry: PxBoxGeometry,
}

impl RigidDynamicActor {
    pub(crate) fn new(rigid_actor: &PhysicsRigidActor, transform: &GlobalTransform) -> Self {
        debug_assert!(rigid_actor.actor_type == RigidActorType::Dynamic);
        match rigid_actor.collision_geometry {
            CollisionGeometry::Box => {
                // default cube is size 0.5 x 0.5 x 0.5
                let extents = transform.scale * 0.25_f32;
                Self {
                    geometry: PxBoxGeometry::new(extents.x, extents.y, extents.z),
                }
            }
            _ => panic!("unsupported geometry"),
        }
    }

    pub(crate) fn add_actor_to_scene(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        scene: &mut ResMut<'_, Owner<PxScene>>,
        transform: &GlobalTransform,
        entity: Entity,
        material: &mut ResMut<'_, Owner<PxMaterial>>,
    ) {
        let transform: PxTransform = transform.compute_matrix().into();
        let mut actor = physics
            .create_rigid_dynamic(
                transform,
                &self.geometry,
                material,
                10_f32,
                PxTransform::default(),
                entity,
            )
            .unwrap();
        actor.set_angular_damping(0.5);
        scene.add_dynamic_actor(actor);
    }
}

#[derive(Component)]
pub(crate) struct RigidStaticActor {
    //geometry: Box<dyn Geometry + Send + Sync>,
    geometry: PxBoxGeometry,
}

impl RigidStaticActor {
    pub(crate) fn new(rigid_actor: &PhysicsRigidActor, transform: &GlobalTransform) -> Self {
        debug_assert!(rigid_actor.actor_type == RigidActorType::Static);
        match rigid_actor.collision_geometry {
            CollisionGeometry::Box => {
                // default cube is size 0.5 x 0.5 x 0.5
                let extents = transform.scale * 0.25_f32;
                Self {
                    geometry: PxBoxGeometry::new(extents.x, extents.y, extents.z),
                }
            }
            _ => panic!("unsupported geometry"),
        }
    }

    pub(crate) fn add_actor_to_scene(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        scene: &mut ResMut<'_, Owner<PxScene>>,
        transform: &GlobalTransform,
        entity: Entity,
        material: &mut ResMut<'_, Owner<PxMaterial>>,
    ) {
        let transform: PxTransform = transform.compute_matrix().into();
        let actor = physics
            .create_rigid_static(
                transform,
                &self.geometry,
                material,
                PxTransform::default(),
                entity,
            )
            .unwrap();
        scene.add_static_actor(actor);
    }
}
