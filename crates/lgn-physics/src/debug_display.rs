use lgn_ecs::prelude::{Query, Res};
use lgn_graphics_data::Color;
use lgn_graphics_renderer::{debug_display::DebugDisplay, resources::DefaultMeshType};
use lgn_math::prelude::Vec3;
use lgn_transform::prelude::GlobalTransform;
use physx::prelude::PxVec3;

use crate::{collision_geometry::CollisionGeometry, physics_options::PhysicsOptions};

pub(crate) fn display_collision_geometry(
    debug_display: Res<'_, DebugDisplay>,
    physics_options: Res<'_, PhysicsOptions>,
    query: Query<'_, '_, (&CollisionGeometry, &GlobalTransform)>,
) {
    if !physics_options.show_collision_geometry {
        return;
    }

    debug_display.create_display_list(|builder| {
        let debug_color = Color::new(0, 255, 51, 255);
        for (collision_geometry, transform) in query.iter() {
            match collision_geometry {
                CollisionGeometry::Box(box_geometry) => {
                    // default cube mesh is 0.5 x 0.5 x 0.5
                    // so x: -0.25..0.25, y: -0.25..0.25, z: -0.25..25
                    let half_extents: PxVec3 = box_geometry.halfExtents.into();
                    let mut scale: Vec3 = half_extents.into();
                    scale /= 0.25;
                    builder.add_default_mesh(
                        &GlobalTransform::identity()
                            .with_translation(transform.translation)
                            .with_scale(scale) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                            .with_rotation(transform.rotation),
                        DefaultMeshType::Cube,
                        debug_color,
                    );
                }
                CollisionGeometry::Capsule(_capsule_geometry) => {
                    builder.add_default_mesh(transform, DefaultMeshType::Cylinder, debug_color);
                }
                CollisionGeometry::ConvexMesh(_convex_mesh_geometry) => {}
                CollisionGeometry::Plane(_plane_geometry) => {
                    builder.add_default_mesh(transform, DefaultMeshType::GroundPlane, debug_color);
                }
                CollisionGeometry::Sphere(sphere_geometry) => {
                    // default sphere mesh has radius of 0.25 (diameter of 0.5)
                    let radius = sphere_geometry.radius;
                    let scale_factor = radius / 0.25;
                    builder.add_default_mesh(
                        &GlobalTransform::identity()
                            .with_translation(transform.translation)
                            .with_scale(Vec3::ONE * scale_factor) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                            .with_rotation(transform.rotation),
                        DefaultMeshType::Sphere,
                        debug_color,
                    );
                }
                CollisionGeometry::TriangleMesh(_triangle_mesh_geometry) => {}
            }
        }
    });

    drop(debug_display);
    drop(physics_options);
    drop(query);
}
