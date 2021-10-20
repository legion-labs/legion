//! Runtime data structs used in the sample-data test

// BEGIN - Legion Labs lints v0.5
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]

use std::{
    any::{Any, TypeId},
    io,
    sync::Arc,
};

use legion_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistry, AssetRegistryOptions, Reference, Resource,
};
use legion_graphics_runtime::Material;
use legion_math::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn add_loaders(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry
        .add_loader::<Entity>()
        .add_loader::<Instance>()
        .add_loader::<Mesh>()
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
    pub parent: Reference<Entity>,
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
        println!("runtime entity \"{}\" loaded", entity.name);

        // activate references
        if let Some(registry) = &self.registry {
            for child in &mut entity.children {
                child.activate(registry);
            }

            for component in &mut entity.components {
                if let Some(visual) = component.downcast_mut::<Visual>() {
                    visual.renderable_geometry.activate(registry);
                } else if let Some(physics) = component.downcast_mut::<Physics>() {
                    physics.collision_geometry.activate(registry);
                }
            }
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
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

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
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
pub struct InstanceLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for InstanceLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        deserialize_bincode_asset::<Instance>(reader)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let instance = asset.downcast_mut::<Instance>().unwrap();
        println!("runtime instance loaded");

        // activate references
        if let Some(registry) = &self.registry {
            instance.original.activate(registry);
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
        println!("runtime mesh loaded");

        // activate references
        if let Some(registry) = &self.registry {
            for sub_mesh in &mut mesh.sub_meshes {
                sub_mesh.material.activate(registry);
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
    pub material: Reference<Material>,
}
