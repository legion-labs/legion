use lgn_graphics_data::Color;

#[component]
struct StaticMeshComponent {
    #[legion(default = 0)]
    pub mesh_id: usize,
    #[legion(default = (255,0,0,255))]
    pub color: Color,
}
