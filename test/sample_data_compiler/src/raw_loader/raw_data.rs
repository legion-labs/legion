use legion_math::prelude::*;
use legion_utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Instance {
    original: String,
    overrides: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    name: String,
    children: Vec<String>,
    parent: Option<String>,
    components: Vec<Component>,
}

#[derive(Serialize, Deserialize)]
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Serialize, Deserialize)]
pub struct Visual {
    renderable_geometry: String,
    shadow_receiver: bool,
    shadow_caster_sun: bool,
    shadow_caster_local: bool,
    gi_contribution: GIContribution,
}

#[derive(Serialize, Deserialize)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    apply_to_children: bool,
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
    fov: f32,
    near: f32,
    far: f32,
    projection_type: ProjectionType,
}

#[derive(Serialize, Deserialize)]
pub struct Light {}

#[derive(Serialize, Deserialize)]
pub enum Component {
    Transform(Transform),
    Visual(Visual),
    GlobalIllumination(GlobalIllumination),
    Navmesh(NavMesh),
    View(View),
    Light(Light),
    Physics(Physics),
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
    dynamic: bool,
    collision_geometry: String,
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    code: String,
    exposed_vars: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct CollisionMaterial {
    impact_script: Option<Script>,
}

#[derive(Serialize, Deserialize)]
pub struct VoxelisationConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMeshLayerConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMesh {
    voxelisation_config: VoxelisationConfig,
    layer_config: Vec<NavMeshLayerConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    dependencies: Vec<String>,
    content_checksum: String,
}
