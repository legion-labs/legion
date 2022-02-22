use lgn_math::prelude::*;

#[resource()]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub positions: Vec<Vec4>,
    pub normals: Vec<Vec4>,
    pub tangents: Vec<Vec4>,
    pub tex_coords: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub colors: Vec<Vec4>,

    #[legion(resource_type = crate::runtime::Material)]
    pub material: Option<ResourcePathId>,
}
