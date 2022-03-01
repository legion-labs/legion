use lgn_graphics_data::Color;
use lgn_graphics_data::DefaultMeshType;

#[component()]
struct DefaultMesh {
    #[legion(default = DefaultMeshType::Cube)]
    pub mesh_type: DefaultMeshType,

    #[legion(default = (255, 0, 0))]
    pub color: Color,
}
