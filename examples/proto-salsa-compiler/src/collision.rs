use std::{fmt::Display, sync::Arc};

use crate::inputs::Inputs;

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

#[salsa::query_group(CollisionStorage)]
pub trait CollisionCompiler: Inputs {
    fn compile_collision(&self, name: Arc<String>) -> AABBCollision;
}

pub fn compile_collision(_db: &dyn CollisionCompiler, raw_data: Arc<String>) -> AABBCollision {
    let split_resources: Vec<&str> = raw_data.split(',').collect();

    AABBCollision {
        // Should handle this parsing much better.
        min_x: split_resources[0].parse::<i64>().unwrap(),
        min_y: split_resources[1].parse::<i64>().unwrap(),
        min_z: split_resources[2].parse::<i64>().unwrap(),
        max_x: split_resources[3].parse::<i64>().unwrap(),
        max_y: split_resources[4].parse::<i64>().unwrap(),
        max_z: split_resources[5].parse::<i64>().unwrap(),
    }
}
