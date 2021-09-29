use legion_app::Plugin;
use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_runtime::{
    manifest::Manifest, AssetDescriptor, AssetId, AssetRegistry, AssetRegistryOptions,
    HandleUntyped,
};
use legion_ecs::prelude::*;
use legion_transform::prelude::*;
use sample_data_compiler::runtime_data::{self};
use std::{
    cmp,
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
struct AssetRegistryState {
    assets: Vec<AssetInfo>,
}

struct AssetInfo {
    id: AssetId,
    handle: HandleUntyped,
    state: AssetState,
    entity: Option<Entity>, // todo: should be enum, to cover other types?
}

impl cmp::PartialOrd for AssetInfo {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl cmp::Ord for AssetInfo {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl cmp::PartialEq for AssetInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl cmp::Eq for AssetInfo {}

#[derive(PartialEq)]
enum AssetState {
    PendingLoad,
    Loaded,
    FailedToLoad,
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
                let registry = registry.create(Box::new(content_store), manifest);

                app.insert_resource(AssetRegistryState::default())
                    .insert_resource(registry)
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
        mut state: ResMut<'_, AssetRegistryState>,
        mut registry: ResMut<'_, AssetRegistry>,
        settings: ResMut<'_, AssetRegistrySettings>,
    ) {
        if let Ok(asset_id) = AssetId::from_str(&settings.root_asset) {
            Self::load_asset(&mut state, &mut registry, asset_id);
        }

        drop(settings);
    }

    fn load_asset(
        state: &mut ResMut<'_, AssetRegistryState>,
        registry: &mut ResMut<'_, AssetRegistry>,
        asset_id: AssetId,
    ) {
        if let Ok(_index) = state
            .assets
            .binary_search_by(|asset_info| asset_info.id.cmp(&asset_id))
        {
            // already in asset list
            println!("New reference to loaded asset: {}", asset_id);
        } else {
            println!("Request load of asset: {}", asset_id);
            state.assets.push(AssetInfo {
                id: asset_id,
                handle: registry.load_untyped(asset_id),
                state: AssetState::PendingLoad,
                entity: None,
            });
            state.assets.sort();
        }
    }

    fn update_registry(mut registry: ResMut<'_, AssetRegistry>) {
        registry.update();
    }

    fn update_assets(
        mut state: ResMut<'_, AssetRegistryState>,
        mut registry: ResMut<'_, AssetRegistry>,
        mut commands: Commands<'_>,
    ) {
        let mut secondary_assets = Vec::new();

        state
            .assets
            .iter_mut()
            .for_each(|asset_info| match asset_info.state {
                AssetState::PendingLoad => {
                    if asset_info.handle.is_loaded(&registry) {
                        match asset_info.id.asset_type() {
                            runtime_data::Entity::TYPE => {
                                if let Some(runtime_entity) =
                                    asset_info.handle.get::<runtime_data::Entity>(&registry)
                                {
                                    let entity = Self::create_entity(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_entity,
                                    );
                                    println!(
                                        "Loaded entity \"{}\", ECS id: {:?}, asset: {}",
                                        runtime_entity.name, entity, asset_info.id
                                    );
                                    asset_info.entity = Some(entity);
                                }
                            }
                            runtime_data::Instance::TYPE => {
                                if let Some(runtime_instance) =
                                    asset_info.handle.get::<runtime_data::Instance>(&registry)
                                {
                                    let instance = Self::create_instance(
                                        &mut commands,
                                        &mut secondary_assets,
                                        runtime_instance,
                                    );
                                    println!(
                                        "Loaded instance, ECS id: {:?}, asset: {}",
                                        instance, asset_info.id
                                    );
                                    asset_info.entity = Some(instance);
                                }
                            }
                            _ => {
                                eprintln!("Unhandled asset loaded: {:?}", asset_info.id);
                            }
                        }
                        asset_info.state = AssetState::Loaded;
                    } else if asset_info.handle.is_err(&registry) {
                        eprintln!("Failed to load asset {}", asset_info.id);
                        asset_info.state = AssetState::FailedToLoad;
                    }
                }
                AssetState::Loaded | AssetState::FailedToLoad => {}
            });

        for asset_id in secondary_assets {
            Self::load_asset(&mut state, &mut registry, asset_id);
        }

        drop(registry);
    }

    fn create_entity(
        commands: &mut Commands<'_>,
        secondary_assets: &mut Vec<AssetId>,
        runtime_entity: &runtime_data::Entity,
    ) -> Entity {
        let mut entity = commands.spawn();

        for component in &runtime_entity.components {
            if let Some(transform) = component.downcast_ref::<runtime_data::Transform>() {
                entity.insert(Transform {
                    translation: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                });
            } else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
                if let Some(geometry) = visual.renderable_geometry {
                    secondary_assets.push(geometry);
                }
            }
            // } else if let Some(gi) = component.downcast_ref::<runtime_data::GlobalIllumination>() {
            // } else if let Some(nav_mesh) = component.downcast_ref::<runtime_data::NavMesh>() {
            // } else if let Some(view) = component.downcast_ref::<runtime_data::View>() {
            // } else if let Some(light) = component.downcast_ref::<runtime_data::Light>() {
            // } else if let Some(physics) = component.downcast_ref::<runtime_data::Physics>() {
        }

        secondary_assets.extend(runtime_entity.children.iter());

        entity.id()
    }

    fn create_instance(
        commands: &mut Commands<'_>,
        secondary_assets: &mut Vec<AssetId>,
        instance: &runtime_data::Instance,
    ) -> Entity {
        let entity = commands.spawn();

        if let Some(original) = instance.original {
            secondary_assets.push(original);
        }

        entity.id()
    }
}

fn read_manifest(manifest_path: impl AsRef<Path>) -> Manifest {
    match File::open(manifest_path) {
        Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
        Err(_e) => Manifest::default(),
    }
}
