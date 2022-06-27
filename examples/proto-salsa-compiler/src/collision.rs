use std::{fmt::Display, sync::Arc};

use crate::{compiler::Compiler, BuildParams};

// Using i64 because float equality doesn't exist in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AABBCollision {
    pub min_x: i64,
    pub min_y: i64,
    pub min_z: i64,
    pub max_x: i64,
    pub max_y: i64,
    pub max_z: i64,
}

impl Default for AABBCollision {
    fn default() -> Self {
        Self {
            min_x: i64::MAX,
            min_y: i64::MAX,
            min_z: i64::MAX,
            max_x: i64::MIN,
            max_y: i64::MIN,
            max_z: i64::MIN,
        }
    }
}

impl AABBCollision {
    pub fn extend(&self, other: &AABBCollision) -> AABBCollision {
        AABBCollision {
            min_x: if self.min_x < other.min_x {
                self.min_x
            } else {
                other.min_x
            },
            min_y: if self.min_y < other.min_y {
                self.min_y
            } else {
                other.min_y
            },
            min_z: if self.min_z < other.min_z {
                self.min_z
            } else {
                other.min_z
            },
            max_x: if self.max_x > other.max_x {
                self.max_x
            } else {
                other.max_x
            },
            max_y: if self.max_y > other.max_y {
                self.max_y
            } else {
                other.max_y
            },
            max_z: if self.max_z > other.max_z {
                self.max_z
            } else {
                other.max_z
            },
        }
    }
}

impl Display for AABBCollision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "minX: {}, minY: {}, minZ: {}, maxX {}, maxY {}, maxZ {}",
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z
        )
    }
}

pub fn compile_aabb(
    _db: &dyn Compiler,
    min_x: Arc<String>,
    min_y: Arc<String>,
    min_z: Arc<String>,
    max_x: Arc<String>,
    max_y: Arc<String>,
    max_z: Arc<String>,
) -> AABBCollision {
    AABBCollision {
        // Should handle this parsing much better.
        min_x: min_x.parse::<i64>().unwrap(),
        min_y: min_y.parse::<i64>().unwrap(),
        min_z: min_z.parse::<i64>().unwrap(),
        max_x: max_x.parse::<i64>().unwrap(),
        max_y: max_y.parse::<i64>().unwrap(),
        max_z: max_z.parse::<i64>().unwrap(),
    }
}

pub fn query_collisions(
    db: &dyn Compiler,
    expressions: String,
    build_params: Arc<BuildParams>,
) -> Arc<Vec<AABBCollision>> {
    let values = db.run(expressions, build_params);

    let mut ret: Vec<AABBCollision> = Vec::new();
    for value in values {
        if let Some(aabb) = value.downcast_ref::<AABBCollision>() {
            println!("{}", aabb);
            ret.push(aabb.clone());
        }
    }
    Arc::new(ret)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{collision::AABBCollision, compiler::Compiler, tests::setup, BuildParams};

    #[test]
    fn compile_aabb() {
        let db = setup();

        let build_params = Arc::new(BuildParams::default());

        let aabb_expression = db.execute_expression("aabb(0,1,2,3,4,5)".to_string(), build_params);

        let aabb = aabb_expression.downcast_ref::<AABBCollision>().unwrap();

        assert_eq!(aabb.min_x, 0);
        assert_eq!(aabb.min_y, 1);
        assert_eq!(aabb.min_z, 2);
        assert_eq!(aabb.max_x, 3);
        assert_eq!(aabb.max_y, 4);
        assert_eq!(aabb.max_z, 5);
    }

    #[test]
    fn test_collisions() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "collisions(read(Car.coll))";

        let aabb_expression = db.execute_expression(expression.to_string(), build_params);

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
    fn collisions_myworld() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        let expression = "collisions(read(MyWorld.entity))";

        let collisions_expression = db.execute_expression(expression.to_string(), build_params);

        let collisions = collisions_expression
            .downcast_ref::<Vec<AABBCollision>>()
            .unwrap();

        assert_eq!(collisions[0].min_x, 5);
        assert_eq!(collisions[0].min_y, 5);
        assert_eq!(collisions[0].min_z, 5);
        assert_eq!(collisions[0].max_x, 10);
        assert_eq!(collisions[0].max_y, 10);
        assert_eq!(collisions[0].max_z, 10);
    }
}
