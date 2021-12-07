


pub struct Float1(f32);
pub struct Float2(glam::Vec2);

pub struct Float3(glam::Vec3);

pub struct Float4(glam::Vec4);

pub struct Float4x4(glam::Mat4);

pub mod prelude {
    pub use crate::Float1;
    pub use crate::Float2;
    pub use crate::Float3;
    pub use crate::Float4;
    pub use crate::Float4x4;
}