use lgn_graphics_data::Color;
use lgn_math::prelude::*;
use lgn_utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub original: String,
    pub overrides: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub children: Vec<String>,
    pub parent: Option<String>,
    pub components: Vec<Component>,
}

/*#[derive(Serialize, Deserialize)]
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}*/

#[derive(Serialize, Deserialize)]
pub struct Visual {
    pub renderable_geometry: Option<String>,
    pub color: Vec3,
    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Serialize, Deserialize)]
pub struct GlobalIllumination {}

#[derive(Serialize, Deserialize)]
pub enum ProjectionType {
    Orthogonal,
    Perspective,
}

#[derive(Serialize, Deserialize)]
pub struct View {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection_type: ProjectionType,
}

#[derive(Serialize, Deserialize)]
pub struct Light {
    pub light_type: sample_data::LightType,
    pub color: Vec3,
    pub radiance: f32,
    pub enabled: bool,
    pub cone_angle: f32,
}

#[derive(Serialize, Deserialize)]
pub struct GltfLoader {
    pub models: Vec<String>,
    pub materials: Vec<String>,
    pub textures: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Component {
    Transform(Transform),
    Visual(Visual),
    GlobalIllumination(GlobalIllumination),
    Navmesh(NavMesh),
    View(View),
    Light(Light),
    GltfLoader(GltfLoader),
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: String,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub albedo: String,
    pub normal: String,
    pub roughness: String,
    pub metalness: String,
    pub base_albedo: Color,
    pub base_metalness: f32,
    pub base_roughness: f32,
    pub reflectance: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub code: String,
    pub exposed_vars: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct CollisionMaterial {
    pub impact_script: Option<Script>,
}

#[derive(Serialize, Deserialize)]
pub struct VoxelisationConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMeshLayerConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMesh {
    pub voxelisation_config: VoxelisationConfig,
    pub layer_config: Vec<NavMeshLayerConfig>,
}
