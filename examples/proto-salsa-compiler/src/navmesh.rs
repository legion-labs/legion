use std::sync::Arc;

use crate::{collision::AABBCollision, compiler::Compiler};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(AABBCollision);

pub fn compile_navmesh(_db: &dyn Compiler, collisions: Arc<Vec<AABBCollision>>) -> Navmesh {
    let all_collisions = collisions
        .iter()
        .fold(AABBCollision::default(), |all, current| all.extend(current));

    Navmesh(all_collisions)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        collision::AABBCollision, compiler::Compiler, navmesh::Navmesh, tests::setup, BuildParams,
    };

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
    fn navmesh_myworld() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "navmesh(collisions(read(MyWorld.entity)))";

        let navmesh_expression = db.execute_expression(expression.to_string(), build_params);

        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 5);
        assert_eq!(navmesh.0.min_y, 5);
        assert_eq!(navmesh.0.min_z, 5);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);
    }

    #[test]
    fn navmesh_add_object() {
        let mut db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "navmesh(collisions(read(MyWorld.entity)))";

        let navmesh_expression =
            db.execute_expression(expression.to_string(), build_params.clone());
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 5);
        assert_eq!(navmesh.0.min_y, 5);
        assert_eq!(navmesh.0.min_z, 5);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);

        db.set_read(
            "MyWorld.entity".to_string(),
            r#"atlas(read(Atlas.atlas));exec(read(Car.coll));exec(read(Tree.coll));aabb(4,4,4,10,10,10)"#.to_string(),
        );

        let navmesh_expression = db.execute_expression(expression.to_string(), build_params);
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 4);
        assert_eq!(navmesh.0.min_y, 4);
        assert_eq!(navmesh.0.min_z, 4);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);
    }

    #[test]
    fn navmesh_remove_object() {
        let mut db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "navmesh(collisions(read(MyWorld.entity)))";

        let navmesh_expression =
            db.execute_expression(expression.to_string(), build_params.clone());
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 5);
        assert_eq!(navmesh.0.min_y, 5);
        assert_eq!(navmesh.0.min_z, 5);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);

        db.set_read(
            "MyWorld.entity".to_string(),
            r#"atlas(read(Atlas.atlas));exec(read(Car.coll))"#.to_string(),
        );

        let navmesh_expression = db.execute_expression(expression.to_string(), build_params);
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 5);
        assert_eq!(navmesh.0.min_y, 5);
        assert_eq!(navmesh.0.min_z, 5);
        assert_eq!(navmesh.0.max_x, 10);
        assert_eq!(navmesh.0.max_y, 10);
        assert_eq!(navmesh.0.max_z, 10);
    }

    #[test]
    fn navmesh_move_object() {
        let mut db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "navmesh(collisions(read(MyWorld.entity)))";

        let navmesh_expression =
            db.execute_expression(expression.to_string(), build_params.clone());
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 5);
        assert_eq!(navmesh.0.min_y, 5);
        assert_eq!(navmesh.0.min_z, 5);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);

        db.set_read(
            "Car.coll".to_string(),
            r#"aabb(6,6,6,15,15,15)"#.to_string(),
        );

        let navmesh_expression = db.execute_expression(expression.to_string(), build_params);
        let navmesh = navmesh_expression.downcast_ref::<Navmesh>().unwrap();

        assert_eq!(navmesh.0.min_x, 6);
        assert_eq!(navmesh.0.min_y, 6);
        assert_eq!(navmesh.0.min_z, 6);
        assert_eq!(navmesh.0.max_x, 50);
        assert_eq!(navmesh.0.max_y, 60);
        assert_eq!(navmesh.0.max_z, 70);
    }
}
