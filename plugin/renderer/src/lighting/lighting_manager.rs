pub struct LightingManager {
    pub num_directional_lights: u32,
    pub num_omnidirectional_lights: u32,
    pub num_spotlights: u32,

    pub specular: bool,
    pub diffuse: bool,
    pub specular_reflection: f32,
    pub diffuse_reflection: f32,
    pub ambient_reflection: f32,
    pub shininess: f32,
}

impl LightingManager {
    pub fn gpu_data(&self) -> Vec<f32> {
        vec![
            f32::from_bits(self.num_directional_lights),
            f32::from_bits(self.num_omnidirectional_lights),
            f32::from_bits(self.num_spotlights),
            f32::from_bits(self.diffuse as u32),
            f32::from_bits(self.specular as u32),
            self.specular_reflection,
            self.diffuse_reflection,
            self.ambient_reflection,
            self.shininess,
        ]
    }
}

impl Default for LightingManager {
    fn default() -> Self {
        Self {
            num_directional_lights: 0,
            num_omnidirectional_lights: 0,
            num_spotlights: 0,

            specular: true,
            diffuse: true,
            specular_reflection: 1.0,
            diffuse_reflection: 1.0,
            ambient_reflection: 0.2,
            shininess: 16.0,
        }
    }
}
