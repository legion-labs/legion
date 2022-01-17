use lgn_graphics_data::Color;
#[component]
struct LightComponent {
    #[legion(default = (255,255,255,255))]
    pub light_color: Color,
}
