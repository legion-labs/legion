use lgn_ecs::prelude::*;
use lgn_transform::prelude::GlobalTransform;
use physx::{
    convex_mesh::ConvexMesh, cooking::PxConvexMeshDesc, foundation::DefaultAllocator, prelude::*,
    traits::Class,
};
use physx_sys::{PxConvexMeshGeometryFlags, PxMeshScale};

use crate::{mesh_scale::MeshScale, runtime, PxMaterial, PxScene, PxShape};

#[derive(Component)]
pub(crate) enum CollisionGeometry {
    Box(PxBoxGeometry),
    Capsule(PxCapsuleGeometry),
    ConvexMesh(PxConvexMeshGeometry),
    //HeightField(PxHeightFieldGeometry),
    Plane(PxPlaneGeometry),
    Sphere(PxSphereGeometry),
    //TriangleMesh(PxTriangleMeshGeometry),
}

// SAFETY: the geometry is created when the physics component are parsed, and then immutable
#[allow(unsafe_code)]
unsafe impl Send for CollisionGeometry {}
#[allow(unsafe_code)]
unsafe impl Sync for CollisionGeometry {}

impl From<&runtime::PhysicsRigidBox> for CollisionGeometry {
    fn from(value: &runtime::PhysicsRigidBox) -> Self {
        Self::Box(PxBoxGeometry::new(
            value.half_extents.x,
            value.half_extents.y,
            value.half_extents.z,
        ))
    }
}

impl From<&runtime::PhysicsRigidCapsule> for CollisionGeometry {
    fn from(value: &runtime::PhysicsRigidCapsule) -> Self {
        Self::Capsule(PxCapsuleGeometry::new(value.radius, value.half_height))
    }
}

impl From<&runtime::PhysicsRigidConvexMesh> for CollisionGeometry {
    fn from(value: &runtime::PhysicsRigidConvexMesh) -> Self {
        let vertices: Vec<PxVec3> = value.vertices.iter().map(|v| (*v).into()).collect();

        let mesh_desc = PxConvexMeshDesc::new();

        let mut mesh = ConvexMesh::default();
        let mesh_scale: PxMeshScale = (&value.scale).into();
        let flags: PxConvexMeshGeometryFlags = ConvexMeshGeometryFlag::TightBounds.into();
        Self::ConvexMesh(PxConvexMeshGeometry::new(&mut mesh, &mesh_scale, flags))
    }
}

impl From<&runtime::PhysicsRigidPlane> for CollisionGeometry {
    fn from(_value: &runtime::PhysicsRigidPlane) -> Self {
        Self::Plane(PxPlaneGeometry::new())
    }
}

impl From<&runtime::PhysicsRigidSphere> for CollisionGeometry {
    fn from(value: &runtime::PhysicsRigidSphere) -> Self {
        Self::Sphere(PxSphereGeometry::new(value.radius))
    }
}

#[allow(unsafe_code)]
unsafe impl Class<PxGeometry> for CollisionGeometry {
    fn as_ptr(&self) -> *const PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_ptr(),
            Self::Capsule(geometry) => geometry.as_ptr(),
            Self::ConvexMesh(geometry) => geometry.as_ptr(),
            // Self::HeightField(geometry) => geometry.as_ptr(),
            Self::Plane(geometry) => geometry.as_ptr(),
            Self::Sphere(geometry) => geometry.as_ptr(),
            // Self::TriangleMesh(geometry) => geometry.as_ptr(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_mut_ptr(),
            Self::Capsule(geometry) => geometry.as_mut_ptr(),
            Self::ConvexMesh(geometry) => geometry.as_mut_ptr(),
            // Self::HeightField(geometry) => geometry.as_mut_ptr(),
            Self::Plane(geometry) => geometry.as_mut_ptr(),
            Self::Sphere(geometry) => geometry.as_mut_ptr(),
            // Self::TriangleMesh(geometry) => geometry.as_mut_ptr(),
        }
    }
}

impl From<&runtime::MeshScale> for PxMeshScale {
    fn from(value: &runtime::MeshScale) -> Self {
        let scale: PxVec3 = value.scale.into();
        let rotation: PxQuat = value.rotation.into();
        Self::new(&scale, &rotation)
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
