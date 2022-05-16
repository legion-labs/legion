use lgn_math::prelude::*;

#[resource()]
#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

#[derive(Clone)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<Vec4>,
    pub tex_coords: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub colors: Vec<Color>,

    #[legion(resource_type = crate::runtime::Material)]
    pub material: Option<ResourcePathId>,
}
