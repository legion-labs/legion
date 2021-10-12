use std::{
    any::{Any, TypeId},
    io,
};

use legion_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistryOptions, Reference, Resource,
};
use legion_graphics_runtime::Material;
use legion_math::prelude::*;
use serde::{Deserialize, Serialize};

pub fn add_loaders(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry
        .add_loader::<Entity>()
        .add_loader::<Instance>()
        .add_loader::<Mesh>()
}

// ------------------ Entity -----------------------------------

#[resource("runtime_entity")]
#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub children: Vec<Reference<Entity>>,
    pub parent: Reference<Entity>,
    pub components: Vec<Box<dyn Component>>,
}

impl Asset for Entity {
    type Loader = EntityLoader;
}

#[derive(Default)]
pub struct EntityLoader {}

impl AssetLoader for EntityLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let deserialize: Result<Entity, Box<bincode::ErrorKind>> =
            bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

#[typetag::serde]
pub trait Component: Any + Send + Sync {}

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
    pub renderable_geometry: Reference<Mesh>,
    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: GIContribution,
}

#[typetag::serde]
impl Component for Visual {}

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
    pub collision_geometry: Reference<Mesh>,
}

#[typetag::serde]
impl Component for Physics {}

// ------------------ Instance  -----------------------------------

#[resource("runtime_instance")]
#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub original: Reference<Entity>,
}

impl Asset for Instance {
    type Loader = InstanceLoader;
}

#[derive(Default)]
pub struct InstanceLoader {}

impl AssetLoader for InstanceLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let deserialize: Result<Instance, Box<bincode::ErrorKind>> =
            bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
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
pub struct MeshLoader {}

impl AssetLoader for MeshLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let deserialize: Result<Mesh, Box<bincode::ErrorKind>> = bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: Reference<Material>,
}
