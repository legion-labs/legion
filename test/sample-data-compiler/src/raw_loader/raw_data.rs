use legion_math::prelude::*;
use legion_utils::HashMap;
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

#[derive(Serialize, Deserialize)]
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Serialize, Deserialize)]
pub struct Visual {
    pub renderable_geometry: String,
    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: GIContribution,
}

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub apply_to_children: bool,
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
pub struct Light {}

#[derive(Serialize, Deserialize)]
pub struct StaticMesh {
    pub mesh_id: usize,
}

#[derive(Serialize, Deserialize)]
pub enum Component {
    Transform(Transform),
    Visual(Visual),
    GlobalIllumination(GlobalIllumination),
    Navmesh(NavMesh),
    View(View),
    Light(Light),
    Physics(Physics),
    StaticMesh(StaticMesh),
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
}

#[derive(Serialize, Deserialize)]
pub struct Physics {
    pub dynamic: bool,
    pub collision_geometry: String,
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
