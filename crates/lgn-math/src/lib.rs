//! Legion Math
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

mod angle;
mod face_toward;
mod geometry;
mod mesh;

pub use angle::*;
pub use face_toward::*;
pub use geometry::*;
pub use glam::*;
pub use mesh::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        BVec2, BVec3, BVec4, EulerRot, FaceToward, IVec2, IVec3, IVec4, Mat3, Mat4, Quat, Rect,
        Size, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
    };
}
