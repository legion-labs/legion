use crate::cgen::cgen_type::LightingData;

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
    pub fn gpu_data(&self) -> LightingData {
        let mut lighting_data = LightingData::default();

        lighting_data.set_num_directional_lights(self.num_directional_lights.into());
        lighting_data.set_num_omni_directional_lights(self.num_omnidirectional_lights.into());
        lighting_data.set_num_spot_lights(self.num_spotlights.into());
        lighting_data.set_diffuse(if self.diffuse { 1 } else { 0 }.into());
        lighting_data.set_specular(if self.specular { 1 } else { 0 }.into());
        lighting_data.set_specular_reflection(self.specular_reflection.into());
        lighting_data.set_diffuse_reflection(self.diffuse_reflection.into());
        lighting_data.set_ambient_reflection(self.ambient_reflection.into());
        lighting_data.set_shininess(self.shininess.into());

        lighting_data
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
            ambient_reflection: 0.0,
            shininess: 16.0,
        }
    }
}
