use std::sync::Arc;

use crate::{collision::AABBCollision, expression::ResourceCompiler, inputs::Inputs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(String);

#[salsa::query_group(NavmeshStorage)]
pub trait NavmeshCompiler: Inputs + ResourceCompiler {
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
