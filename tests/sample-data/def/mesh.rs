use lgn_math::prelude::*;

#[resource()]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,

    #[legion(resource_type = lgn_graphics_data::runtime::Material)]
    pub material: Option<ResourcePathId>,
}
