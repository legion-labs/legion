use lgn_ecs::prelude::{Component, Res, ResMut};
use lgn_math::prelude::Vec3;
use lgn_tracing::info;
use physx::{
    cooking::{
        ConvexMeshCookingResult, PxConvexMeshDesc, PxCooking, PxTriangleMeshDesc,
        TriangleMeshCookingResult,
    },
    foundation::DefaultAllocator,
    prelude::{
        BoxGeometry, CapsuleGeometry, ConvexMeshGeometry, Owner, PhysicsFoundation, PlaneGeometry,
        PxBoxGeometry, PxCapsuleGeometry, PxConvexMeshGeometry, PxGeometry, PxPlaneGeometry,
        PxSphereGeometry, PxTriangleMeshGeometry, PxVec3, SphereGeometry, TriangleMeshGeometry,
    },
    traits::Class,
};
use physx_sys::{PxConvexFlag, PxConvexMeshGeometryFlags, PxMeshGeometryFlags, PxMeshScale};
use thiserror::Error;

use crate::{runtime, PxShape};

#[derive(Component)]
pub(crate) enum CollisionGeometry {
    Box(PxBoxGeometry),
    Capsule(PxCapsuleGeometry),
    ConvexMesh(PxConvexMeshGeometry),
    //HeightField(PxHeightFieldGeometry),
    Plane(PxPlaneGeometry),
    Sphere(PxSphereGeometry),
    TriangleMesh(PxTriangleMeshGeometry),
}

// SAFETY: the geometry is created when the physics component are parsed, and then immutable
#[allow(unsafe_code)]
unsafe impl Send for CollisionGeometry {}
#[allow(unsafe_code)]
unsafe impl Sync for CollisionGeometry {}

impl Drop for CollisionGeometry {
    fn drop(&mut self) {
        info!("drop CollisionGeometry");
    }
}

pub(crate) type ConvertResult = Result<CollisionGeometry, ConvertError>;

pub(crate) trait ConvertToCollisionGeometry {
    fn convert(
        &self,
        scale: &Vec3,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult;
}

#[derive(Error, Debug)]
pub(crate) enum ConvertError {
    #[error("generic conversion failure")]
    Failure,
    #[error("invalid convex/triangle mesh descriptor")]
    InvalidDescriptor,
    #[error("large triangle")]
    LargeTriangle,
    #[error("polygons limit reached")]
    PolygonsLimitReached,
    #[error("zero-area test failed, area too small so cannot produce a valid hull")]
    ZeroAreaTestFailed,
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidBox {
    fn convert(
        &self,
        scale: &Vec3,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        Ok(CollisionGeometry::Box(PxBoxGeometry::new(
            self.half_extents.x * scale.x,
            self.half_extents.y * scale.y,
            self.half_extents.z * scale.z,
        )))
    }
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidCapsule {
    fn convert(
        &self,
        _scale: &Vec3,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        // TODO: take scale into account (average?)
        Ok(CollisionGeometry::Capsule(PxCapsuleGeometry::new(
            self.radius,
            self.half_height,
        )))
    }
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidConvexMesh {
    #[allow(clippy::fn_to_numeric_cast_with_truncation)]
    fn convert(
        &self,
        scale: &Vec3,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        let vertices: Vec<PxVec3> = self.vertices.iter().map(|v| (*v).into()).collect();
        let mut mesh_desc = PxConvexMeshDesc::new();
        mesh_desc.obj.points.data = vertices.as_ptr().cast::<std::ffi::c_void>();
        mesh_desc.obj.points.count = vertices.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.flags.mBits = PxConvexFlag::eCOMPUTE_CONVEX as u16;

        match cooking.create_convex_mesh(physics.physics_mut(), &mesh_desc) {
            ConvexMeshCookingResult::Success(mut convex_mesh) => {
                let mut mesh_scale = self.scale;
                mesh_scale.scale *= *scale;
                let mesh_scale: PxMeshScale = mesh_scale.into();
                let flags = PxConvexMeshGeometryFlags { mBits: 0 };
                let geometry = CollisionGeometry::ConvexMesh(PxConvexMeshGeometry::new(
                    convex_mesh.as_mut(),
                    &mesh_scale,
                    flags,
                ));

                // prevent cooked mesh from being dropped immediately
                #[allow(clippy::mem_forget)]
                std::mem::forget(convex_mesh);

                Ok(geometry)
            }
            ConvexMeshCookingResult::Failure => Err(ConvertError::Failure),
            ConvexMeshCookingResult::InvalidDescriptor => Err(ConvertError::InvalidDescriptor),
            ConvexMeshCookingResult::PolygonsLimitReached => {
                Err(ConvertError::PolygonsLimitReached)
            }
            ConvexMeshCookingResult::ZeroAreaTestFailed => Err(ConvertError::ZeroAreaTestFailed),
        }
    }
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidPlane {
    fn convert(
        &self,
        _scale: &Vec3,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        Ok(CollisionGeometry::Plane(PxPlaneGeometry::new()))
    }
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidSphere {
    fn convert(
        &self,
        _scale: &Vec3,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        // TODO: take scale into account (average?)
        Ok(CollisionGeometry::Sphere(PxSphereGeometry::new(
            self.radius,
        )))
    }
}

impl ConvertToCollisionGeometry for runtime::PhysicsRigidTriangleMesh {
    fn convert(
        &self,
        scale: &Vec3,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> ConvertResult {
        let vertices: Vec<PxVec3> = self.vertices.iter().map(|v| (*v).into()).collect();
        let mut mesh_desc = PxTriangleMeshDesc::new();
        mesh_desc.obj.points.data = vertices.as_ptr().cast::<std::ffi::c_void>();
        mesh_desc.obj.points.count = vertices.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;

        if !cooking.validate_triangle_mesh(&mesh_desc) {
            return Err(ConvertError::InvalidDescriptor);
        }

        match cooking.create_triangle_mesh(physics.physics_mut(), &mesh_desc) {
            TriangleMeshCookingResult::Success(mut triangle_mesh) => {
                let mut mesh_scale = self.scale;
                mesh_scale.scale *= *scale;
                let mesh_scale: PxMeshScale = mesh_scale.into();
                let flags = PxMeshGeometryFlags { mBits: 0 };
                let geometry = CollisionGeometry::TriangleMesh(PxTriangleMeshGeometry::new(
                    triangle_mesh.as_mut(),
                    &mesh_scale,
                    flags,
                ));

                // prevent cooked mesh from being dropped immediately
                #[allow(clippy::mem_forget)]
                std::mem::forget(triangle_mesh);

                Ok(geometry)
            }
            TriangleMeshCookingResult::Failure => Err(ConvertError::Failure),
            TriangleMeshCookingResult::InvalidDescriptor => Err(ConvertError::InvalidDescriptor),
            TriangleMeshCookingResult::LargeTriangle => Err(ConvertError::LargeTriangle),
        }
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
            Self::TriangleMesh(geometry) => geometry.as_ptr(),
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
            Self::TriangleMesh(geometry) => geometry.as_mut_ptr(),
        }
    }
}
