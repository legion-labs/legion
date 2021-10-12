//! The asset registry plugin provides loading of runtime assets.
//!

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

mod asset_entities;
mod asset_handles;
mod loading_states;
mod settings;

use asset_entities::AssetToEntityMap;
use asset_handles::AssetHandles;
use loading_states::{AssetLoadingStates, LoadingState};
pub use settings::AssetRegistrySettings;

use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryOptions, Reference, Resource, ResourceId,
};
use legion_ecs::prelude::*;
use legion_transform::prelude::*;
use sample_data_compiler::runtime_data;
use std::{any::Any, fs::File, path::Path};

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(mut settings) = app.world.get_resource_mut::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(settings.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = read_manifest(&settings.game_manifest);
                if settings.assets_to_load.is_empty() {
                    settings.assets_to_load = manifest.resources().copied().collect();
                }

                let mut registry = AssetRegistryOptions::new();
                registry = runtime_data::add_loaders(registry);
                registry = legion_graphics_runtime::add_loaders(registry);
                let registry = registry
                    .add_device_cas(Box::new(content_store), manifest)
                    .create();

                app.insert_resource(registry)
                    .insert_resource(AssetLoadingStates::default())
                    .insert_resource(AssetHandles::default())
                    .insert_resource(AssetToEntityMap::default())
                    .add_startup_system(Self::setup)
                    .add_system(Self::update_registry)
                    .add_system(Self::update_assets);
            } else {
                eprintln!(
                    "Unable to open content storage in {:?}",
                    settings.content_store_addr
                );
            }
        } else {
            eprintln!("Missing AssetRegistrySettings resource, must add to app");
        }
    }
}

impl AssetRegistryPlugin {
    fn setup(
        mut registry: ResMut<'_, AssetRegistry>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        settings: ResMut<'_, AssetRegistrySettings>,
    ) {
        for asset_id in &settings.assets_to_load {
            Self::load_asset(
                &mut registry,
                &mut asset_loading_states,
                &mut asset_handles,
                *asset_id,
            );
        }

        drop(settings);
    }

    fn load_asset(
        registry: &mut ResMut<'_, AssetRegistry>,
        asset_loading_states: &mut ResMut<'_, AssetLoadingStates>,
        asset_handles: &mut ResMut<'_, AssetHandles>,
        asset_id: ResourceId,
    ) {
        if let Some(_handle) = asset_handles.get(asset_id) {
            // already in asset list
        } else {
            asset_loading_states.insert(asset_id, LoadingState::Pending);
            asset_handles.insert(asset_id, registry.load_untyped(asset_id));
        }
    }

    fn update_registry(mut registry: ResMut<'_, AssetRegistry>) {
        registry.update();
    }

    #[allow(clippy::too_many_lines)]
    fn update_assets(
        mut registry: ResMut<'_, AssetRegistry>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        mut asset_to_entity_map: ResMut<'_, AssetToEntityMap>,
        mut commands: Commands<'_, '_>,
    ) {
        let mut secondary_assets = SecondaryAssets::default();

        for (asset_id, loading_state) in asset_loading_states.iter_mut() {
            match loading_state {
                LoadingState::Pending => {
                    let handle = asset_handles.get(*asset_id).unwrap();
                    if handle.is_loaded(&registry) {
                        match asset_id.ty() {
                            runtime_data::Entity::TYPE => {
                                if let Some(runtime_entity) =
                                    handle.get::<runtime_data::Entity>(&registry)
                                {
                                    let entity = runtime_data::Entity::create_in_ecs(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_entity,
                                        &asset_to_entity_map,
                                    );

                                    if let Some(entity_id) = entity {
                                        asset_to_entity_map.insert(*asset_id, entity_id);

                                        println!(
                                            "Loaded runtime entity \"{}\", ECS id: {:?}, asset: {}",
                                            runtime_entity.name, entity_id, asset_id
                                        );
                                    }
                                }
                            }
                            runtime_data::Instance::TYPE => {
                                if let Some(runtime_instance) =
                                    handle.get::<runtime_data::Instance>(&registry)
                                {
                                    let instance = runtime_data::Instance::create_in_ecs(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_instance,
                                        &asset_to_entity_map,
                                    );

                                    if let Some(entity_id) = instance {
                                        asset_to_entity_map.insert(*asset_id, entity_id);

                                        println!(
                                            "Loaded runtime instance, ECS id: {:?}, asset: {}",
                                            entity_id, asset_id
                                        );
                                    }
                                }
                            }
                            legion_graphics_runtime::Material::TYPE => {
                                if let Some(runtime_material) =
                                    handle.get::<legion_graphics_runtime::Material>(&registry)
                                {
                                    legion_graphics_runtime::Material::create_in_ecs(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_material,
                                        &asset_to_entity_map,
                                    );

                                    println!("Loaded runtime material, asset: {}", asset_id);
                                }
                            }
                            runtime_data::Mesh::TYPE => {
                                if let Some(runtime_mesh) =
                                    handle.get::<runtime_data::Mesh>(&registry)
                                {
                                    runtime_data::Mesh::create_in_ecs(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_mesh,
                                        &asset_to_entity_map,
                                    );

                                    println!("Loaded runtime mesh, asset: {}", asset_id);
                                }
                            }
                            legion_graphics_runtime::Texture::TYPE => {
                                if let Some(runtime_texture) =
                                    handle.get::<legion_graphics_runtime::Texture>(&registry)
                                {
                                    legion_graphics_runtime::Texture::create_in_ecs(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_texture,
                                        &asset_to_entity_map,
                                    );

                                    println!("Loaded runtime texture, asset: {}", asset_id);
                                }
                            }
                            _ => {
                                eprintln!(
                                    "Unhandled runtime type: {}, asset: {}",
                                    asset_id.ty(),
                                    asset_id
                                );
                            }
                        }

                        *loading_state = LoadingState::Loaded;
                    } else if handle.is_err(&registry) {
                        eprintln!("Failed to load runtime asset {}", asset_id);
                        *loading_state = LoadingState::Failed;
                    }
                }
                LoadingState::Loaded | LoadingState::Failed => {}
            }
        }

        // request load for assets referenced by
        for asset_id in secondary_assets {
            Self::load_asset(
                &mut registry,
                &mut asset_loading_states,
                &mut asset_handles,
                asset_id,
            );
        }

        drop(registry);
    }
}

trait AssetToECS {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        secondary_assets: &mut SecondaryAssets,
        asset: &Self,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity>;
}

impl AssetToECS for runtime_data::Entity {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        secondary_assets: &mut SecondaryAssets,
        runtime_entity: &Self,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let mut entity = commands.spawn();

        let mut transform_inserted = false;
        for component in &runtime_entity.components {
            if let Some(transform) = component.downcast_ref::<runtime_data::Transform>() {
                entity.insert(Transform {
                    translation: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                });
                transform_inserted = true;
            } else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
                secondary_assets.push(&visual.renderable_geometry);
            }
            // } else if let Some(gi) = component.downcast_ref::<runtime_data::GlobalIllumination>() {
            // } else if let Some(nav_mesh) = component.downcast_ref::<runtime_data::NavMesh>() {
            // } else if let Some(view) = component.downcast_ref::<runtime_data::View>() {
            // } else if let Some(light) = component.downcast_ref::<runtime_data::Light>() {
            // } else if let Some(physics) = component.downcast_ref::<runtime_data::Physics>() {
        }

        if !transform_inserted {
            entity.insert(Transform::identity());
        }
        entity.insert(GlobalTransform::identity());

        // load child entities
        secondary_assets.extend(runtime_entity.children.iter());

        // parent, if it exists, must already be loaded since parents load their children
        let parent = if let Reference::Passive(parent) = runtime_entity.parent {
            asset_to_entity_map.get(parent)
        } else {
            None
        };

        if let Some(parent) = parent {
            entity.insert(Parent(parent));
        }

        let entity_id = entity.id();

        if let Some(parent) = parent {
            commands.entity(parent).push_children(&[entity_id]);
        }

        Some(entity_id)
    }
}

impl AssetToECS for runtime_data::Instance {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        secondary_assets: &mut SecondaryAssets,
        instance: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let entity = commands.spawn();

        secondary_assets.push(&instance.original);

        Some(entity.id())
    }
}

impl AssetToECS for legion_graphics_runtime::Material {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        secondary_assets: &mut SecondaryAssets,
        material: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        secondary_assets.push(&material.albedo);
        secondary_assets.push(&material.normal);
        secondary_assets.push(&material.roughness);
        secondary_assets.push(&material.metalness);

        None
    }
}

impl AssetToECS for runtime_data::Mesh {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        secondary_assets: &mut SecondaryAssets,
        mesh: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        for sub_mesh in &mesh.sub_meshes {
            secondary_assets.push(&sub_mesh.material);
        }

        None
    }
}

impl AssetToECS for legion_graphics_runtime::Texture {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        _secondary_assets: &mut SecondaryAssets,
        _texture: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        None
    }
}

#[derive(Default)]
struct SecondaryAssets(Vec<ResourceId>);

impl SecondaryAssets {
    fn push<T>(&mut self, asset_id: &Reference<T>)
    where
        T: Any + Resource,
    {
        if let Reference::Passive(asset_id) = asset_id {
            self.0.push(*asset_id);
        }
    }
}

impl<'a> Extend<&'a ResourceId> for SecondaryAssets {
    fn extend<T: IntoIterator<Item = &'a ResourceId>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl IntoIterator for SecondaryAssets {
    type Item = ResourceId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
