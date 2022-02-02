use lgn_graphics_data::Color;
use lgn_graphics_data::DefaultMeshType;

#[component()]
struct StaticMesh {
    #[legion(default=DefaultMeshType::Cube)]
    pub mesh_id: DefaultMeshType,

    #[legion(default=(255,0,0))]
    pub color: Color,

    #[legion(resource_type = crate::runtime::Mesh)]
    pub mesh: Option<ResourcePathId>,
}
