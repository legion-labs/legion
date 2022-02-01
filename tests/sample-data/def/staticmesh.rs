use lgn_graphics_data::Color;

#[component()]
struct StaticMesh {
    pub mesh_id: usize,

    #[legion(default=(255,0,0))]
    pub color: Color,

    #[legion(resource_type = crate::runtime::Mesh)]
    pub mesh: Option<ResourcePathId>,
}
