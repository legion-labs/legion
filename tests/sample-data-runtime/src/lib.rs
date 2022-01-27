//! Runtime data structs used in the sample-data test

// crate-specific lint exceptions:
//#![allow()]

use std::{
    any::{Any, TypeId},
    io,
    path::PathBuf,
    sync::Arc,
};

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistry, AssetRegistryOptions, Reference, Resource,
};
use lgn_graphics_runtime::Material;
use lgn_math::prelude::*;
use lgn_tracing::info;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn add_loaders(registry: &mut AssetRegistryOptions) {
    registry
        .add_loader_mut::<Entity>()
        .add_loader_mut::<Instance>()
        .add_loader_mut::<Mesh>()
        .add_loader_mut::<Script>();
}

fn deserialize_bincode_asset<T>(reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>>
where
    T: DeserializeOwned + Any + Send + Sync,
{
    let deserialize: Result<T, Box<bincode::ErrorKind>> = bincode::deserialize_from(reader);
    match deserialize {
        Ok(asset) => Ok(Box::new(asset)),
        Err(err) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        )),
    }
}

// ------------------ Entity -----------------------------------

#[resource("runtime_entity")]
#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub children: Vec<Reference<Entity>>,
    pub parent: Option<Reference<Entity>>,
    pub components: Vec<Box<dyn Component>>,
}

impl Asset for Entity {
    type Loader = EntityLoader;
}

#[derive(Default)]
pub struct EntityLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for EntityLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        deserialize_bincode_asset::<Entity>(reader)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let entity = asset.downcast_mut::<Entity>().unwrap();
        info!("runtime entity \"{}\" loaded", entity.name);

        // activate references
        if let Some(registry) = &self.registry {
            for child in &mut entity.children {
                child.activate(registry);
            }

            for component in &mut entity.components {
                component.activate_references(registry);
            }
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}

#[typetag::serde]
pub trait Component: Any + Send + Sync {
    fn activate_references(&mut self, _registry: &AssetRegistry) {}
}

/// Note: Based on impl of dyn Any
impl dyn Component {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Component>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Component>(&self) -> Option<&T> {
        if self.is::<T>() {
            #[allow(unsafe_code)]
            unsafe {
                Some(&*((self as *const dyn Component).cast::<T>()))
            }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`,
    /// or `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Component>(&mut self) -> Option<&mut T> {
        #[allow(unsafe_code)]
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Component).cast::<T>()) }
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub apply_to_children: bool,
}

#[typetag::serde]
impl Component for Transform {}

#[derive(Serialize, Deserialize)]
pub struct Visual {
    pub renderable_geometry: Option<Reference<Mesh>>,
    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: GIContribution,
}

#[typetag::serde]
impl Component for Visual {
    fn activate_references(&mut self, registry: &AssetRegistry) {
        if let Some(geometry) = &mut self.renderable_geometry {
            geometry.activate(registry);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Serialize, Deserialize)]
pub struct GlobalIllumination {}

#[typetag::serde]
impl Component for GlobalIllumination {}

#[derive(Serialize, Deserialize)]
pub struct NavMesh {
    pub voxelisation_config: VoxelisationConfig,
    pub layer_config: Vec<NavMeshLayerConfig>,
}

#[typetag::serde]
impl Component for NavMesh {}

#[derive(Serialize, Deserialize)]
pub struct VoxelisationConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMeshLayerConfig {}

#[derive(Serialize, Deserialize)]
pub struct View {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection_type: ProjectionType,
}

#[typetag::serde]
impl Component for View {}

#[derive(Serialize, Deserialize)]
pub enum ProjectionType {
    Orthogonal,
    Perspective,
}

#[derive(Serialize, Deserialize)]
pub struct Light {}

#[typetag::serde]
impl Component for Light {}

#[derive(Serialize, Deserialize)]
pub struct Physics {
    pub dynamic: bool,
    pub collision_geometry: Option<Reference<Mesh>>,
}

#[typetag::serde]
impl Component for Physics {
    fn activate_references(&mut self, registry: &AssetRegistry) {
        if let Some(geometry) = &mut self.collision_geometry {
            geometry.activate(registry);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StaticMesh {
    pub mesh_id: usize,
}

#[typetag::serde]
impl Component for StaticMesh {}

// ------------------ Script -----------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub enum ScriptType {
    Mun,
    Rune,
    Rhai,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ScriptPayload {
    None,
    LibPath(PathBuf),
    ContainedScript(String),
}

#[derive(Serialize, Deserialize)]
pub struct ScriptComponent {
    pub script_type: ScriptType,
    pub input_values: Vec<String>,
    pub entry_fn: String,
    pub script: Option<Reference<Script>>,
    pub payload: ScriptPayload,
}

#[typetag::serde]
impl Component for ScriptComponent {}

#[resource("runtime_script")]
#[derive(Serialize, Deserialize)]
pub struct Script {
    pub data: Vec<u8>,
}

impl Asset for Script {
    type Loader = ScriptLoader;
}

#[derive(Default)]
pub struct ScriptLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for ScriptLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        //deserialize_bincode_asset::<Script>(reader)
        let mut data = vec![];
        reader.read_to_end(&mut data).unwrap();
        Ok(Box::new(Script { data }))
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let _script = asset.downcast_mut::<Script>().unwrap();
        println!("runtime script loaded");

        // activate references
        /*if let Some(registry) = &self.registry {
            if let Some(original) = &mut instance.original {
                original.activate(registry);
            }
        }*/
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}

// ------------------ Instance  -----------------------------------

#[resource("runtime_instance")]
#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub original: Option<Reference<Entity>>,
}

impl Asset for Instance {
    type Loader = InstanceLoader;
}

#[derive(Default)]
pub struct InstanceLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for InstanceLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        deserialize_bincode_asset::<Instance>(reader)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let instance = asset.downcast_mut::<Instance>().unwrap();
        info!("runtime instance loaded");

        // activate references
        if let Some(registry) = &self.registry {
            if let Some(original) = &mut instance.original {
                original.activate(registry);
            }
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}

// ------------------ Mesh -----------------------------------

#[resource("runtime_mesh")]
#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

impl Asset for Mesh {
    type Loader = MeshLoader;
}

#[derive(Default)]
pub struct MeshLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for MeshLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        deserialize_bincode_asset::<Mesh>(reader)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let mesh = asset.downcast_mut::<Mesh>().unwrap();
        info!("runtime mesh loaded");

        // activate references
        if let Some(registry) = &self.registry {
            for sub_mesh in &mut mesh.sub_meshes {
                if let Some(material) = &mut sub_mesh.material {
                    material.activate(registry);
                }
            }
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: Option<Reference<Material>>,
}
