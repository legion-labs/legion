use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::{Query, Res};
use lgn_graphics_renderer::{debug_display::DebugDisplay, resources::DefaultMeshType};
use lgn_math::prelude::Vec3;
use lgn_transform::prelude::{GlobalTransform, Transform};
use physx::prelude::PxVec3;

use crate::{collision_geometry::CollisionGeometry, physics_options::PhysicsOptions};

pub(crate) fn display_collision_geometry(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    physics_options: Res<'_, PhysicsOptions>,
    query: Query<'_, '_, (&CollisionGeometry, &GlobalTransform)>,
) {
    if !physics_options.show_collision_geometry {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for (collision_geometry, transform) in query.iter() {
                match collision_geometry {
                    CollisionGeometry::Box(box_geometry) => {
                        // default cube mesh is 0.5 x 0.5 x 0.5
                        // so x: -0.25..0.25, y: -0.25..0.25, z: -0.25..25
                        let half_extents: PxVec3 = box_geometry.halfExtents.into();
                        let mut scale: Vec3 = half_extents.into();
                        scale /= 0.25;
                        builder.add_mesh(
                            Transform::identity()
                                .with_translation(transform.translation)
                                .with_scale(scale) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                                .with_rotation(transform.rotation)
                                .compute_matrix(),
                            DefaultMeshType::Cube as u32,
                            Vec3::new(0.8, 0.8, 0.3),
                        );
                    }
                    CollisionGeometry::Capsule(_capsule_geometry) => {
                        builder.add_mesh(
                            transform.compute_matrix(),
                            DefaultMeshType::Cylinder as u32,
                            Vec3::new(0.8, 0.8, 0.3),
                        );
                    }
                    CollisionGeometry::ConvexMesh(_convex_mesh_geometry) => {}
                    CollisionGeometry::Plane(_plane_geometry) => {
                        builder.add_mesh(
                            transform.compute_matrix(),
                            DefaultMeshType::GroundPlane as u32,
                            Vec3::new(0.8, 0.8, 0.3),
                        );
                    }
                    CollisionGeometry::Sphere(sphere_geometry) => {
                        // default sphere mesh has radius of 0.25 (diameter of 0.5)
                        let radius = sphere_geometry.radius;
                        let scale_factor = radius / 0.25;
                        builder.add_mesh(
                            Transform::identity()
                                .with_translation(transform.translation)
                                .with_scale(Vec3::ONE * scale_factor) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                                .with_rotation(transform.rotation)
                                .compute_matrix(),
                            DefaultMeshType::Sphere as u32,
                            Vec3::new(0.8, 0.8, 0.3),
                        );
                    }
                    CollisionGeometry::TriangleMesh(_triangle_mesh_geometry) => {}
                }
            }
        });
    });

    drop(debug_display);
    drop(bump_allocator_pool);
    drop(physics_options);
    drop(query);
}
