use std::marker::PhantomData;

use legion_data_runtime::{Asset, AssetLoader, AssetRegistryOptions, AssetType};
use legion_math::prelude::*;
use legion_utils::HashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Asset, Serialize, Deserialize)]
struct Instance {
    original: String,
    overrides: HashMap<String, String>,
}

#[derive(Asset, Serialize, Deserialize)]
pub struct Entity {
    name: String,
    children: Vec<String>,
    parent: Option<String>,
    components: Vec<Component>,
}

#[derive(Asset, Serialize, Deserialize)]
enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Asset, Serialize, Deserialize)]
struct Visual {
    renderable_geometry: String,
    shadow_receiver: bool,
    shadow_caster_sun: bool,
    shadow_caster_local: bool,
    gi_contribution: GIContribution,
}

#[derive(Asset, Serialize, Deserialize)]
struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    apply_to_children: bool,
}

#[derive(Asset, Serialize, Deserialize)]
struct GlobalIllumination {}

#[derive(Asset, Serialize, Deserialize)]
enum ProjectionType {
    Orthogonal,
    Perspective,
}

#[derive(Asset, Serialize, Deserialize)]
struct View {
    fov: f32,
    near: f32,
    far: f32,
    projection_type: ProjectionType,
}

#[derive(Asset, Serialize, Deserialize)]
struct Light {}

#[derive(Asset, Serialize, Deserialize)]
enum Component {
    Transform(Transform),
    Visual(Visual),
    GlobalIllumination(GlobalIllumination),
    Navmesh(NavMesh),
    View(View),
    Light(Light),
    Physics(Physics),
}

#[derive(Asset, Serialize, Deserialize, Debug, Default, PartialEq)]
struct SubMesh {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u16>,
    material: String,
}

#[derive(Asset, Serialize, Deserialize)]
struct Mesh {
    sub_meshes: Vec<SubMesh>,
}

#[derive(Asset, Serialize, Deserialize)]
struct Material {
    albedo: String,
    normal: String,
    roughness: String,
    metalness: String,
}

#[derive(Asset, Serialize, Deserialize)]
struct Physics {
    dynamic: bool,
    collision_geometry: String,
}

#[derive(Asset, Serialize, Deserialize)]
struct Script {
    code: String,
    exposed_vars: HashMap<String, String>,
}

#[derive(Asset, Serialize, Deserialize)]
struct CollisionMaterial {
    impact_script: Option<Script>,
}

#[derive(Asset, Serialize, Deserialize)]
struct VoxelisationConfig {}

#[derive(Asset, Serialize, Deserialize)]
struct NavMeshLayerConfig {}

#[derive(Asset, Serialize, Deserialize)]
struct NavMesh {
    voxelisation_config: VoxelisationConfig,
    layer_config: Vec<NavMeshLayerConfig>,
}

#[derive(Asset, Serialize, Deserialize)]
struct Metadata {
    name: String,
    dependencies: Vec<String>,
    content_checksum: String,
}

pub struct RONAssetCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> RONAssetCreator<T> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> AssetLoader for RONAssetCreator<T>
where
    T: Asset + Send + Sync + DeserializeOwned,
{
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let deserialize: Result<T, ron::Error> = ron::de::from_reader(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(_ron_err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unable to read RON data",
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}

pub const ENTITY_TYPE_ID: AssetType = AssetType::new(b"ron_entity");
pub const MATERIAL_TYPE_ID: AssetType = AssetType::new(b"ron_material");
pub const MESH_TYPE_ID: AssetType = AssetType::new(b"ron_mesh");
pub const SUB_MESH_TYPE_ID: AssetType = AssetType::new(b"ron_sub_mesh");

pub fn register_asset_loaders(asset_options: AssetRegistryOptions) -> AssetRegistryOptions {
    asset_options
        .add_creator(ENTITY_TYPE_ID, Box::new(RONAssetCreator::<Entity>::new()))
        .add_creator(
            MATERIAL_TYPE_ID,
            Box::new(RONAssetCreator::<Material>::new()),
        )
        .add_creator(MESH_TYPE_ID, Box::new(RONAssetCreator::<Mesh>::new()))
        .add_creator(
            SUB_MESH_TYPE_ID,
            Box::new(RONAssetCreator::<SubMesh>::new()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_mesh_ron_roundtrip() {
        let sub_mesh = SubMesh::default();
        let serialized = ron::ser::to_string(&sub_mesh).unwrap();
        dbg!(&serialized);
        let deserialized: SubMesh = ron::de::from_str(&serialized).unwrap();
        assert_eq!(sub_mesh, deserialized);
    }
}
