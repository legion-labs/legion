use lgn_ecs::prelude::*;
use lgn_transform::prelude::*;
use physx::{foundation::DefaultAllocator, prelude::*, traits::Class};

use crate::{
    runtime::{PhysicsRigidBox, PhysicsRigidSphere},
    PxMaterial, PxScene, PxShape,
};

#[derive(Component)]
pub(crate) struct BoxCollisionGeometry(PxBoxGeometry);

impl BoxCollisionGeometry {
    pub(crate) fn new(rigid_box: &PhysicsRigidBox) -> Self {
        Self(PxBoxGeometry::new(
            rigid_box.half_extents.x,
            rigid_box.half_extents.y,
            rigid_box.half_extents.z,
        ))
    }
}

#[allow(unsafe_code)]
unsafe impl Class<PxGeometry> for BoxCollisionGeometry {
    fn as_ptr(&self) -> *const PxGeometry {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut PxGeometry {
        self.0.as_mut_ptr()
    }
}

#[derive(Component)]
pub(crate) struct SphereCollisionGeometry(PxSphereGeometry);

impl SphereCollisionGeometry {
    pub(crate) fn new(rigid_sphere: &PhysicsRigidSphere) -> Self {
        Self(PxSphereGeometry::new(rigid_sphere.radius))
    }
}

#[allow(unsafe_code)]
unsafe impl Class<PxGeometry> for SphereCollisionGeometry {
    fn as_ptr(&self) -> *const PxGeometry {
        self.0.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut PxGeometry {
        self.0.as_mut_ptr()
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
