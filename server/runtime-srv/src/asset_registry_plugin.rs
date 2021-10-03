use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryOptions, HandleUntyped, Resource, ResourceId,
};
use legion_ecs::prelude::*;
use legion_transform::prelude::*;
use sample_data_compiler::runtime_data;
use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct AssetRegistrySettings {
    content_store_addr: PathBuf,
    game_manifest: PathBuf,
    root_asset: String,
}

impl AssetRegistrySettings {
    pub fn new(
        content_store_addr: impl AsRef<Path>,
        game_manifest: impl AsRef<Path>,
        root_asset: &str,
    ) -> Self {
        Self {
            content_store_addr: content_store_addr.as_ref().to_owned(),
            game_manifest: game_manifest.as_ref().to_owned(),
            root_asset: root_asset.to_string(),
        }
    }
}

#[derive(Default)]
struct AssetLoadingStates(BTreeMap<ResourceId, LoadingState>);

enum LoadingState {
    Pending,
    Loaded,
    Failed,
}

#[derive(Default)]
struct AssetHandles(BTreeMap<ResourceId, HandleUntyped>);

impl AssetHandles {
    fn get(&self, asset_id: ResourceId) -> Option<&HandleUntyped> {
        self.0.get(&asset_id)
    }
}

#[derive(Default)]
struct AssetToEntityMap(BTreeMap<ResourceId, Entity>);

impl AssetToEntityMap {
    fn get(&self, asset_id: ResourceId) -> Option<Entity> {
        self.0.get(&asset_id).copied()
    }

    fn insert(&mut self, asset_id: ResourceId, entity: Entity) {
        self.0.insert(asset_id, entity);
    }
}

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
                let registry = registry.create(Box::new(content_store), manifest);

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
        if let Ok(asset_id) = ResourceId::from_str(&settings.root_asset) {
            Self::load_asset(
                &mut registry,
                &mut asset_loading_states,
                &mut asset_handles,
                asset_id,
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
            println!("New reference to loaded asset: {}", asset_id);
        } else {
            println!("Request load of asset: {}", asset_id);
            asset_loading_states
                .0
                .insert(asset_id, LoadingState::Pending);
            asset_handles
                .0
                .insert(asset_id, registry.load_untyped(asset_id));
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
        mut commands: Commands<'_>,
    ) {
        let mut secondary_assets = Vec::new();

        for (asset_id, loading_state) in &mut asset_loading_states.0 {
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

    fn add_secondary_asset(secondary_assets: &mut Vec<ResourceId>, asset_id: Option<ResourceId>) {
        if let Some(asset_id) = asset_id {
            secondary_assets.push(asset_id);
        }
    }

    fn create_entity(
        commands: &mut Commands<'_>,
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
                Self::add_secondary_asset(secondary_assets, visual.renderable_geometry);
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
        let parent = if let Some(parent) = runtime_entity.parent {
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
        commands: &mut Commands<'_>,
        secondary_assets: &mut Vec<ResourceId>,
        instance: &runtime_data::Instance,
    ) -> Entity {
        let entity = commands.spawn();

        Self::add_secondary_asset(secondary_assets, instance.original);

        entity.id()
    }

    fn create_material(
        _commands: &mut Commands<'_>,
        secondary_assets: &mut Vec<ResourceId>,
        material: &legion_graphics_runtime::Material,
    ) {
        Self::add_secondary_asset(secondary_assets, material.albedo);
        Self::add_secondary_asset(secondary_assets, material.normal);
        Self::add_secondary_asset(secondary_assets, material.roughness);
        Self::add_secondary_asset(secondary_assets, material.metalness);
    }

    fn create_mesh(
        _commands: &mut Commands<'_>,
        secondary_assets: &mut Vec<ResourceId>,
        mesh: &runtime_data::Mesh,
    ) {
        for sub_mesh in &mesh.sub_meshes {
            Self::add_secondary_asset(secondary_assets, sub_mesh.material);
        }
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
