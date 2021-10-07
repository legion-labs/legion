//! The asset registry plugin provides loading of runtime assets.
//!

// BEGIN - Legion Labs lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// END - Legion Labs standard lints v0.4
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
use std::{any::Any, fs::File, path::Path, str::FromStr};

#[derive(Default)]
pub struct AssetRegistryPlugin {}

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut legion_app::App) {
        if let Some(settings) = app.world.get_resource::<AssetRegistrySettings>() {
            let content_store_addr = ContentStoreAddr::from(settings.content_store_addr.clone());
            if let Some(content_store) = HddContentStore::open(content_store_addr) {
                let manifest = read_manifest(&settings.game_manifest);

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
        // check if root asset specified
        if let Some(root_asset) = &settings.root_asset {
            if let Ok(asset_id) = ResourceId::from_str(root_asset) {
                Self::load_asset(
                    &mut registry,
                    &mut asset_loading_states,
                    &mut asset_handles,
                    asset_id,
                );
            } else {
                eprintln!("Unable to parse root asset: {}", root_asset);
            }
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
            println!("New reference to loaded asset: {}", asset_id);
        } else {
            println!("Request load of asset: {}", asset_id);
            asset_loading_states.insert(asset_id, LoadingState::Pending);
            asset_handles.insert(asset_id, registry.load_untyped(asset_id));
        }
    }

    fn update_registry(mut registry: ResMut<'_, AssetRegistry>) {
        registry.update();
    }

    fn update_assets(
        mut registry: ResMut<'_, AssetRegistry>,
        mut asset_loading_states: ResMut<'_, AssetLoadingStates>,
        mut asset_handles: ResMut<'_, AssetHandles>,
        mut asset_to_entity_map: ResMut<'_, AssetToEntityMap>,
        mut commands: Commands<'_, '_>,
    ) {
        let mut secondary_assets = Vec::new();

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
                                    let entity = Self::create_entity(
                                        &mut commands,
                                        &asset_to_entity_map,
                                        &mut secondary_assets,
                                        runtime_entity,
                                    );

                                    asset_to_entity_map.insert(*asset_id, entity);

                                    println!(
                                        "Loaded entity \"{}\", ECS id: {:?}, asset: {}",
                                        runtime_entity.name, entity, asset_id
                                    );
                                }
                            }
                            runtime_data::Instance::TYPE => {
                                if let Some(runtime_instance) =
                                    handle.get::<runtime_data::Instance>(&registry)
                                {
                                    let instance = Self::create_instance(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_instance,
                                    );

                                    asset_to_entity_map.insert(*asset_id, instance);

                                    println!(
                                        "Loaded instance, ECS id: {:?}, asset: {}",
                                        instance, asset_id
                                    );
                                }
                            }
                            legion_graphics_runtime::Material::TYPE => {
                                if let Some(runtime_material) =
                                    handle.get::<legion_graphics_runtime::Material>(&registry)
                                {
                                    Self::create_material(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_material,
                                    );

                                    println!("Loaded material, asset: {}", asset_id);
                                }
                            }
                            runtime_data::Mesh::TYPE => {
                                if let Some(runtime_mesh) =
                                    handle.get::<runtime_data::Mesh>(&registry)
                                {
                                    Self::create_mesh(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_mesh,
                                    );

                                    println!("Loaded mesh, asset: {}", asset_id);
                                }
                            }
                            legion_graphics_runtime::Texture::TYPE => {
                                if let Some(runtime_texture) =
                                    handle.get::<legion_graphics_runtime::Texture>(&registry)
                                {
                                    Self::create_texture(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_texture,
                                    );

                                    println!("Loaded texture, asset: {}", asset_id);
                                }
                            }
                            _ => {
                                eprintln!("Unhandled type: {}, asset: {}", asset_id.ty(), asset_id);
                            }
                        }

                        *loading_state = LoadingState::Loaded;
                    } else if handle.is_err(&registry) {
                        eprintln!("Failed to load asset {}", asset_id);
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

    fn add_secondary_asset<T>(secondary_assets: &mut Vec<ResourceId>, asset_id: &Reference<T>)
    where
        T: Any + Resource,
    {
        if let Reference::Passive(asset_id) = asset_id {
            secondary_assets.push(*asset_id);
        }
    }

    fn create_entity(
        commands: &mut Commands<'_, '_>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        secondary_assets: &mut Vec<ResourceId>,
        runtime_entity: &runtime_data::Entity,
    ) -> Entity {
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
                Self::add_secondary_asset(secondary_assets, &visual.renderable_geometry);
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

        entity_id
    }

    fn create_instance(
        commands: &mut Commands<'_, '_>,
        secondary_assets: &mut Vec<ResourceId>,
        instance: &runtime_data::Instance,
    ) -> Entity {
        let entity = commands.spawn();

        Self::add_secondary_asset(secondary_assets, &instance.original);

        entity.id()
    }

    fn create_material(
        _commands: &mut Commands<'_, '_>,
        secondary_assets: &mut Vec<ResourceId>,
        material: &legion_graphics_runtime::Material,
    ) {
        Self::add_secondary_asset(secondary_assets, &material.albedo);
        Self::add_secondary_asset(secondary_assets, &material.normal);
        Self::add_secondary_asset(secondary_assets, &material.roughness);
        Self::add_secondary_asset(secondary_assets, &material.metalness);
    }

    fn create_mesh(
        _commands: &mut Commands<'_, '_>,
        secondary_assets: &mut Vec<ResourceId>,
        mesh: &runtime_data::Mesh,
    ) {
        for sub_mesh in &mesh.sub_meshes {
            Self::add_secondary_asset(secondary_assets, &sub_mesh.material);
        }
    }

    fn create_texture(
        _commands: &mut Commands<'_, '_>,
        _secondary_assets: &mut Vec<ResourceId>,
        _texture: &legion_graphics_runtime::Texture,
    ) {
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
