use lgn_math::prelude::*;

#[resource()]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}

pub struct SubMesh {
    pub positions: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec4>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub indices: Option<Vec<u32>>,
    pub colors: Option<Vec<Vec4>>,

    #[legion(resource_type = crate::runtime::Material)]
    pub material: Option<ResourcePathId>,
}
