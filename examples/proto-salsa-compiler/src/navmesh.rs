use std::sync::Arc;

use crate::{
    collision::AABBCollision,
    compiler::{AnyEq, Compiler},
    BuildParams,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(AABBCollision);

pub fn query_collisions(
    db: &dyn Compiler,
    expressions: String,
    build_params: Arc<BuildParams>,
) -> Vec<AABBCollision> {
    let values = db.compile_entity(expressions, build_params);

    let mut ret: Vec<AABBCollision> = Vec::new();
    for value in values {
        if let Some(aabb) = value.downcast_ref::<AABBCollision>() {
            ret.push(aabb.clone());
        }
    }
    ret
}

pub fn compile_navmesh(_db: &dyn Compiler, collisions: Arc<Vec<AABBCollision>>) -> Navmesh {
    let all_collisions = collisions
        .iter()
        .fold(AABBCollision::default(), |all, current| all.extend(current));

    Navmesh(all_collisions)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{collision::AABBCollision, compiler::Compiler, tests::setup, BuildParams};

    #[test]
    fn navmesh_simple() {
        let db = setup();

        let navmesh = db.compile_navmesh(Arc::new(vec![AABBCollision {
            min_x: 0,
            min_y: 1,
            min_z: 2,
            max_x: 11,
            max_y: 12,
            max_z: 13,
        }]));

        assert_eq!(navmesh.0.min_x, 0);
        assert_eq!(navmesh.0.min_y, 1);
        assert_eq!(navmesh.0.min_z, 2);
        assert_eq!(navmesh.0.max_x, 11);
        assert_eq!(navmesh.0.max_y, 12);
        assert_eq!(navmesh.0.max_z, 13);
    }

    #[test]
    fn navmesh_two_collisions() {
        let db = setup();

        let navmesh = db.compile_navmesh(Arc::new(vec![
            AABBCollision {
                min_x: 5,
                min_y: 6,
                min_z: 7,
                max_x: 11,
                max_y: 12,
                max_z: 13,
            },
            AABBCollision {
                min_x: 0,
                min_y: 1,
                min_z: 2,
                max_x: 8,
                max_y: 9,
                max_z: 10,
            },
        ]));

        assert_eq!(navmesh.0.min_x, 0);
        assert_eq!(navmesh.0.min_y, 1);
        assert_eq!(navmesh.0.min_z, 2);
        assert_eq!(navmesh.0.max_x, 11);
        assert_eq!(navmesh.0.max_y, 12);
        assert_eq!(navmesh.0.max_z, 13);
    }

    #[test]
    fn test_collisions() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "query_collisions(read(Car.coll))";

        let aabb_expression = db
            .execute_expression(expression.to_string(), build_params)
            .unwrap();

        let aabb = aabb_expression
            .downcast_ref::<Vec<AABBCollision>>()
            .unwrap();

        assert_eq!(aabb[0].min_x, 5);
        assert_eq!(aabb[0].min_y, 5);
        assert_eq!(aabb[0].min_z, 5);
        assert_eq!(aabb[0].max_x, 10);
        assert_eq!(aabb[0].max_y, 10);
        assert_eq!(aabb[0].max_z, 10);
    }

    #[test]
    fn navmesh_add_object() {}

    #[test]
    fn navmesh_remove_object() {}

    #[test]
    fn navmesh_move_object() {}
}
