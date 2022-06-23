use std::sync::Arc;

use crate::{collision::AABBCollision, compiler::Compiler};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(String);

pub fn query_collisions(_db: &dyn Compiler, _quadrant: Arc<AABBCollision>) -> Vec<AABBCollision> {
    vec![]
}

pub fn compile_navmesh(db: &dyn Compiler, quadrant: Arc<AABBCollision>) -> Navmesh {
    let collisions = db.query_collisions(quadrant);

    let all_collisions = collisions
        .iter()
        .fold(AABBCollision::default(), |all, current| all.extend(current));

    Navmesh(format!("Navmesh with collisions {}", &all_collisions))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{collision::AABBCollision, compiler::Compiler, tests::setup};

    #[test]
    fn navmesh_add_object() {
        let db = setup();

        db.compile_navmesh(Arc::new(AABBCollision {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 10,
            max_y: 10,
            max_z: 10,
        }));
    }

    #[test]
    fn navmesh_remove_object() {
        let db = setup();

        db.compile_navmesh(Arc::new(AABBCollision {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 10,
            max_y: 10,
            max_z: 10,
        }));
    }

    #[test]
    fn navmesh_move_object() {}
}
