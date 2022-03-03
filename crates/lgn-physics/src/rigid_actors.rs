use lgn_ecs::prelude::*;
use lgn_transform::prelude::GlobalTransform;
use physx::{
    cooking::PxCooking,
    cooking::{ConvexMeshCookingResult, PxConvexMeshDesc},
    foundation::DefaultAllocator,
    prelude::*,
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

pub(crate) trait ConvertToGeometry {
    fn convert(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry;
}

impl ConvertToGeometry for runtime::PhysicsRigidBox {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry {
        CollisionGeometry::Box(PxBoxGeometry::new(
            self.half_extents.x,
            self.half_extents.y,
            self.half_extents.z,
        ))
    }
}

impl ConvertToGeometry for runtime::PhysicsRigidCapsule {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry {
        CollisionGeometry::Capsule(PxCapsuleGeometry::new(self.radius, self.half_height))
    }
}

impl ConvertToGeometry for runtime::PhysicsRigidConvexMesh {
    #[allow(clippy::fn_to_numeric_cast_with_truncation)]
    fn convert(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry {
        let vertices: Vec<PxVec3> = self.vertices.iter().map(|v| (*v).into()).collect();
        let mut mesh_desc = PxConvexMeshDesc::new();
        mesh_desc.obj.points.data = vertices.as_ptr().cast::<std::ffi::c_void>();
        mesh_desc.obj.points.count = vertices.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3> as u32;
        assert!(cooking.validate_convex_mesh(&mesh_desc));

        let cooking_result = cooking.create_convex_mesh(physics.physics_mut(), &mesh_desc);

        match cooking_result {
            ConvexMeshCookingResult::Success(mut convex_mesh) => {
                let mesh_scale: PxMeshScale = (&self.scale).into();
                let flags: PxConvexMeshGeometryFlags = ConvexMeshGeometryFlag::TightBounds.into();
                CollisionGeometry::ConvexMesh(PxConvexMeshGeometry::new(
                    &mut convex_mesh,
                    &mesh_scale,
                    flags,
                ))
            }
            _ => {
                panic!("mesh cooking failed");
            }
        }
    }
}

impl ConvertToGeometry for runtime::PhysicsRigidPlane {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry {
        CollisionGeometry::Plane(PxPlaneGeometry::new())
    }
}

impl ConvertToGeometry for runtime::PhysicsRigidSphere {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionGeometry {
        CollisionGeometry::Sphere(PxSphereGeometry::new(self.radius))
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
