use lgn_math::prelude::*;

#[resource()]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<Vec3>,
    pub tex_coords: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub colors: Vec<Color>,

    #[legion(resource_type = crate::runtime::Material)]
    pub material: Option<ResourcePathId>,
}
