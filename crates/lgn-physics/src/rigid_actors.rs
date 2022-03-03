use lgn_ecs::prelude::*;
use lgn_transform::prelude::*;
use physx::{foundation::DefaultAllocator, prelude::*, traits::Class};

use crate::{
    runtime::{PhysicsRigidBox, PhysicsRigidCapsule, PhysicsRigidPlane, PhysicsRigidSphere},
    PxMaterial, PxScene, PxShape,
};

#[derive(Component)]
pub(crate) enum CollisionGeometry {
    Box(PxBoxGeometry),
    Capsule(PxCapsuleGeometry),
    Plane(PxPlaneGeometry),
    Sphere(PxSphereGeometry),
}

impl From<&PhysicsRigidBox> for CollisionGeometry {
    fn from(value: &PhysicsRigidBox) -> Self {
        Self::Box(PxBoxGeometry::new(
            value.half_extents.x,
            value.half_extents.y,
            value.half_extents.z,
        ))
    }
}

impl From<&PhysicsRigidCapsule> for CollisionGeometry {
    fn from(value: &PhysicsRigidCapsule) -> Self {
        Self::Capsule(PxCapsuleGeometry::new(value.radius, value.half_height))
    }
}

impl From<&PhysicsRigidPlane> for CollisionGeometry {
    fn from(_value: &PhysicsRigidPlane) -> Self {
        Self::Plane(PxPlaneGeometry::new())
    }
}

impl From<&PhysicsRigidSphere> for CollisionGeometry {
    fn from(value: &PhysicsRigidSphere) -> Self {
        Self::Sphere(PxSphereGeometry::new(value.radius))
    }
}

#[allow(unsafe_code)]
unsafe impl Class<PxGeometry> for CollisionGeometry {
    fn as_ptr(&self) -> *const PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_ptr(),
            Self::Capsule(geometry) => geometry.as_ptr(),
            Self::Plane(geometry) => geometry.as_ptr(),
            Self::Sphere(geometry) => geometry.as_ptr(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_mut_ptr(),
            Self::Capsule(geometry) => geometry.as_mut_ptr(),
            Self::Plane(geometry) => geometry.as_mut_ptr(),
            Self::Sphere(geometry) => geometry.as_mut_ptr(),
        }
    }
}

pub(crate) fn add_dynamic_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) {
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
        .unwrap();
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
        .unwrap();
    scene.add_static_actor(actor);
}
