use crate::{collision::AABBCollision, expression::ResourceCompiler, inputs::Inputs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Navmesh(String);

#[salsa::query_group(NavmeshStorage)]
pub trait NavmeshCompiler: Inputs + ResourceCompiler {
    fn query_collisions(&self, quadrant: AABBCollision) -> Vec<AABBCollision>;
    fn compile_navmesh(&self, quadrant: AABBCollision) -> Navmesh;
}

fn query_collisions(db: &dyn NavmeshCompiler, quadrant: AABBCollision) -> Vec<AABBCollision> {
    vec![]
}

fn compile_navmesh(db: &dyn NavmeshCompiler, quadrant: AABBCollision) -> Navmesh {
    let collisions = db.query_collisions(quadrant);
    Navmesh("Navmesh with collisions {} {}".to_string())
}
