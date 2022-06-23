use std::sync::Arc;

use crate::{collision::AABBCollision, inputs::Inputs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(String);

#[salsa::query_group(NavmeshStorage)]
pub trait NavmeshCompiler: Inputs {
    fn query_collisions(&self, quadrant: Arc<AABBCollision>) -> Vec<AABBCollision>;
    fn compile_navmesh(&self, quadrant: Arc<AABBCollision>) -> Navmesh;
}

fn query_collisions(
    _db: &dyn NavmeshCompiler,
    _quadrant: Arc<AABBCollision>,
) -> Vec<AABBCollision> {
    vec![]
}

fn compile_navmesh(db: &dyn NavmeshCompiler, quadrant: Arc<AABBCollision>) -> Navmesh {
    let collisions = db.query_collisions(quadrant);

    let all_collisions = collisions
        .iter()
        .fold(AABBCollision::default(), |all, current| all.extend(current));

    Navmesh(format!("Navmesh with collisions {}", &all_collisions))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{collision::AABBCollision, tests::setup};

    use super::NavmeshCompiler;

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
