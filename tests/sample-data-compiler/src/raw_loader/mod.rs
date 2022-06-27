mod raw_data;
mod raw_to_offline;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::Provider;
use lgn_data_offline::{Project, ResourcePathName, SourceResource};
use lgn_data_runtime::prelude::*;
use lgn_graphics_data::{
    offline::Gltf,
    offline::Psd,
    offline::{Png, Visual},
};
use lgn_math::Vec3;
use lgn_source_control::{BranchName, RepositoryIndex, RepositoryName};
use lgn_tracing::{error, info};
use lgn_utils::DefaultHasher;
use sample_data::offline as offline_data;
use serde::de::DeserializeOwned;

use self::raw_to_offline::FromRaw;

pub async fn build_offline(
    root_folder: impl AsRef<Path>,
    repository_index: impl RepositoryIndex,
    repository_name: &RepositoryName,
    branch_name: &BranchName,
    source_control_content_provider: Arc<Provider>,
    incremental: bool,
) -> Project {
    let raw_dir = {
        if let Ok(entries) = root_folder.as_ref().read_dir() {
            let mut raw_dir = entries
                .flatten()
                .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
            raw_dir.next().map(|d| d.path())
        } else {
            None
        }
    };

    let (mut project, _resources) = setup_project(
        repository_index,
        repository_name,
        branch_name,
        source_control_content_provider.clone(),
    )
    .await;

    if let Some(raw_dir) = raw_dir {
        let file_paths = find_files(
            &raw_dir,
            &["ent", "ins", "mat", "psd", "png", "gltf", "glb"],
        );

        let raw_checksum = {
            let mut hasher = DefaultHasher::new();
            for file in &file_paths {
                let meta = std::fs::metadata(file).unwrap();
                meta.modified().unwrap().hash(&mut hasher);
            }
            hasher.finish()
        };

        let version_file_path = root_folder.as_ref().join("VERSION");
        let generated_checksum = {
            if !version_file_path.exists() {
                None
            } else {
                std::fs::read_to_string(version_file_path.as_path())
                    .map_or(None, |version| version.parse::<u64>().ok())
            }
        };

        if let Some(generated_checksum) = generated_checksum {
            if generated_checksum == raw_checksum {
                info!("Skipping Project Generation");
                return project;
            }
        }

        if !incremental {
            std::fs::remove_file(version_file_path.as_path()).unwrap_or_default();
        }

        // cleanup data from source control before we generate new data.
        if !incremental {
            let all_resources = project.resource_list().await;
            if !all_resources.is_empty() {
                for type_id in all_resources {
                    project.delete_resource(type_id).await.unwrap();
                }

                project.commit("cleanup resources").await.unwrap();
            }
        }

        let file_paths_guids = file_paths
            .iter()
            .map(|s| {
                let mut p = s.clone();
                p.set_extension(s.extension().unwrap().to_str().unwrap().to_owned() + ".guid");
                ResourceId::from_str(&fs::read_to_string(p).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();

        let in_resources = file_paths
            .iter()
            .map(|s| path_to_resource_name(s))
            .zip(file_paths_guids)
            .collect::<Vec<_>>();

        let resource_ids = create_or_find_default(&file_paths, &in_resources, &mut project).await;

        info!("Created resources: {:#?}", project);

        for (i, path) in file_paths.iter().enumerate() {
            let resource_name = &in_resources[i].0;
            let resource_id = *resource_ids.get(resource_name).unwrap();
            match path.extension().unwrap().to_str().unwrap() {
                "ent" => {
                    load_ron_resource::<raw_data::Entity, offline_data::Entity>(
                        resource_id,
                        resource_name,
                        path,
                        &resource_ids,
                        &mut project,
                    )
                    .await;

                    if let Ok(entity) = project
                        .load_resource::<offline_data::Entity>(resource_id)
                        .await
                    {
                        if let Some(parent_id) = entity
                            .parent
                            .as_ref()
                            .map(lgn_data_runtime::ResourcePathId::source_resource)
                        {
                            let mut new_name =
                                lgn_data_offline::get_meta(entity.as_ref()).name.clone();
                            new_name.replace_parent_info(Some(parent_id), None);
                            if let Err(err) = project.rename_resource(resource_id, &new_name).await
                            {
                                panic!("Failed to rename {}: {}", resource_id, err);
                            }
                        }
                    }
                }
                "ins" => {
                    load_ron_resource::<raw_data::Instance, offline_data::Instance>(
                        resource_id,
                        resource_name,
                        path,
                        &resource_ids,
                        &mut project,
                    )
                    .await;
                }
                "mat" => {
                    load_ron_resource::<raw_data::Material, lgn_graphics_data::offline::Material>(
                        resource_id,
                        resource_name,
                        path,
                        &resource_ids,
                        &mut project,
                    )
                    .await;
                }
                "psd" => {
                    load_psd_resource(
                        resource_id,
                        resource_name,
                        path,
                        &mut project,
                        &source_control_content_provider,
                    )
                    .await;
                }
                "png" => {
                    load_png_resource(
                        resource_id,
                        resource_name,
                        path,
                        &mut project,
                        &source_control_content_provider,
                    )
                    .await;
                }
                "gltf" | "glb" => {
                    load_gltf_resource(
                        resource_id,
                        resource_name,
                        path,
                        &mut project,
                        &source_control_content_provider,
                    )
                    .await;
                }
                _ => panic!(),
            }

            info!("Loaded: {}. id: {}", resource_name, resource_id);
        }

        project.commit("sample data generation").await.unwrap();

        let mut version_file = std::fs::File::create(version_file_path).unwrap();
        version_file
            .write_all(raw_checksum.to_string().as_bytes())
            .unwrap();
    } else {
        error!(
            "did not find a 'raw' sub-directory in {}",
            root_folder.as_ref().display()
        );
    }

    project
}

async fn setup_project(
    repository_index: impl RepositoryIndex,
    repository_name: &RepositoryName,
    branch_name: &BranchName,
    source_control_content_provider: Arc<Provider>,
) -> (Project, Arc<AssetRegistry>) {
    // create/load project
    let project = Project::new(
        &repository_index,
        repository_name,
        branch_name,
        source_control_content_provider,
    )
    .await
    .unwrap();

    let mut registry = AssetRegistryOptions::new();
    lgn_graphics_data::register_types(&mut registry);
    sample_data::register_types(&mut registry);
    generic_data::register_types(&mut registry);

    (project, registry.create().await)
}

fn ext_to_resource_kind(ext: &str) -> (&str, ResourceType) {
    match ext {
        "ent" => (offline_data::Entity::TYPENAME, offline_data::Entity::TYPE),
        "ins" => (
            offline_data::Instance::TYPENAME,
            offline_data::Instance::TYPE,
        ),
        "mat" => (
            lgn_graphics_data::offline::Material::TYPENAME,
            lgn_graphics_data::offline::Material::TYPE,
        ),
        "psd" => (
            lgn_graphics_data::offline::Psd::TYPENAME,
            lgn_graphics_data::offline::Psd::TYPE,
        ),
        "png" => (
            lgn_graphics_data::offline::Png::TYPENAME,
            lgn_graphics_data::offline::Png::TYPE,
        ),
        "gltf" => (
            lgn_graphics_data::offline::Gltf::TYPENAME,
            lgn_graphics_data::offline::Gltf::TYPE,
        ),
        _ => panic!(),
    }
}

/// Creates resources for all `file_paths` containing default values (empty
/// content).
///
/// The content of resources is loaded later.
///
/// This is done because we need to assign `ResourceId` for all resources before
/// we load them in order to resolve references from a `ResourcePathName`
/// (/path/to/resource) to `ResourceId` (125463453).
async fn create_or_find_default(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
) -> HashMap<ResourcePathName, ResourceTypeAndId> {
    let mut ids = HashMap::<ResourcePathName, ResourceTypeAndId>::default();
    build_resource_from_raw(file_paths, in_resources, project, &mut ids).await;
    ids
}

async fn build_resource_from_raw(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    for (i, path) in file_paths.iter().enumerate() {
        let name = &in_resources[i].0;
        let (_kind_name, kind) = ext_to_resource_kind(path.extension().unwrap().to_str().unwrap());

        let id = {
            if let Ok(id) = project.find_resource(name).await {
                id
            } else {
                let id = ResourceTypeAndId {
                    kind,
                    id: in_resources[i].1,
                };

                if project.exists(id).await {
                    project.delete_resource(id).await.unwrap();
                }

                let mut new_resource = kind.new_instance();
                let meta = lgn_data_offline::get_meta_mut(new_resource.as_mut());
                meta.name = name.clone();
                meta.type_id = id;

                project
                    .add_resource_with_id(id, new_resource.as_ref())
                    .await
                    .unwrap();
                id
            }
        };
        ids.insert(name.clone(), id);
    }
}

fn path_to_resource_name(path: &Path) -> ResourcePathName {
    let mut found = false;
    let name = path
        .iter()
        .filter_map(|component| {
            let was_found = found;
            if !found && component == OsStr::new("raw") {
                found = true;
            }
            if was_found {
                let mut s = String::from("/");
                s.push_str(&component.to_owned().into_string().unwrap());
                Some(s)
            } else {
                None
            }
        })
        .collect::<String>();
    ResourcePathName::from(name)
}

fn find_files(raw_dir: impl AsRef<Path>, extensions: &[&str]) -> Vec<PathBuf> {
    let dir = raw_dir.as_ref();

    let mut files = vec![];

    for entry in dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut find_files(&path, extensions));
        } else if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            //
            // include only supported extensions
            //
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }

    files
}

async fn load_ron_resource<RawType, OfflineType>(
    resource_id: ResourceTypeAndId,
    name: &ResourcePathName,
    file: &Path,
    references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    project: &mut Project,
) -> Option<ResourceTypeAndId>
where
    RawType: DeserializeOwned,
    OfflineType: Resource + ResourceDescriptor + FromRaw<RawType> + Send + 'static,
{
    if let Ok(f) = File::open(file) {
        let reader = BufReader::new(f);
        let raw_data: RawType = ron::de::from_reader(reader).unwrap();

        // convert raw to offline
        let mut offline_data = OfflineType::from_raw(raw_data, references);
        let meta = lgn_data_offline::get_meta_mut(&mut offline_data);
        meta.type_id = resource_id;
        meta.name = name.into();

        project
            .save_resource(resource_id, &offline_data)
            .await
            .unwrap();
        Some(resource_id)
    } else {
        None
    }
}

async fn load_psd_resource(
    resource_id: ResourceTypeAndId,
    name: &ResourcePathName,
    file: &Path,
    project: &mut Project,
    source_control_content_provider: &Arc<Provider>,
) -> Option<ResourceTypeAndId> {
    let raw_data = fs::read(file).ok()?;
    let content_id = source_control_content_provider
        .write(&raw_data)
        .await
        .unwrap();

    let mut resource = Psd::new_with_id(name.as_str(), resource_id);
    resource.content_id = content_id.to_string();

    project.save_resource(resource_id, &resource).await.unwrap();
    Some(resource_id)
}

async fn load_png_resource(
    resource_id: ResourceTypeAndId,
    name: &ResourcePathName,
    file: &Path,
    project: &mut Project,
    source_control_content_provider: &Arc<Provider>,
) -> Option<ResourceTypeAndId> {
    let raw_data = fs::read(file).ok()?;
    let content_id = source_control_content_provider
        .write(&raw_data)
        .await
        .unwrap();

    let mut resource = Png::new_with_id(name.as_str(), resource_id);
    resource.content_id = content_id.to_string();

    project.save_resource(resource_id, &resource).await.unwrap();
    Some(resource_id)
}

async fn load_gltf_resource(
    resource_id: ResourceTypeAndId,
    name: &ResourcePathName,
    file: &Path,
    project: &mut Project,
    source_control_content_provider: &Arc<Provider>,
) -> Option<ResourceTypeAndId> {
    lgn_tracing::info!("Loading Gltf {}", name);
    let raw_data = fs::read(file).ok()?;
    lgn_tracing::info!("Uploading raw gltf to content store");
    let content_id = source_control_content_provider
        .write(&raw_data)
        .await
        .unwrap();

    let mut resource = Gltf::new_with_id(name.as_str(), resource_id);
    resource.content_id = content_id.to_string();
    project.save_resource(resource_id, &resource).await.unwrap();

    lgn_tracing::info!("Parsing Gltf");
    let gltf = gltf::Gltf::from_slice_without_validation(&raw_data).unwrap();
    lgn_tracing::info!("Creating hierarchy");
    {
        let root_name = String::from("Root");
        let root_id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: {
                let mut hasher = DefaultHasher::new();
                resource_id.hash(&mut hasher);
                root_name.hash(&mut hasher);
                let id = hasher.finish();
                ResourceId::from_raw(u128::from(id) | (u128::from(id) << 64))
            },
        };

        let mut root_entity = sample_data::offline::Entity::new_with_id(
            &format!("/!{}/{}", resource_id, root_name),
            root_id,
        );
        root_entity.components.push(Box::new(offline_data::Name {
            name: root_name.clone(),
        }));
        root_entity
            .components
            .push(Box::new(sample_data::offline::Transform::default()));

        if !project.exists(root_id).await {
            project
                .add_resource_with_id(root_id, &root_entity)
                .await
                .unwrap();
        }

        for (idx, node) in gltf.document.nodes().enumerate() {
            let child_name = node.name().map_or(idx.to_string(), Into::into);
            let child_id = ResourceTypeAndId {
                kind: sample_data::offline::Entity::TYPE,
                id: {
                    let mut hasher = DefaultHasher::new();
                    resource_id.hash(&mut hasher);
                    child_name.hash(&mut hasher);
                    let id = hasher.finish();
                    ResourceId::from_raw(u128::from(id) | (u128::from(id) << 64))
                },
            };

            root_entity
                .children
                .push(ResourcePathId::from(child_id).push(sample_data::runtime::Entity::TYPE));

            let mut child = sample_data::offline::Entity::new_with_id(
                &format!("/!{}/{}", root_id, child_name),
                child_id,
            );
            child.parent =
                Some(ResourcePathId::from(root_id).push(sample_data::runtime::Entity::TYPE));

            child
                .components
                .push(Box::new(sample_data::offline::Name { name: child_name }));

            let (position, rotation, scale) = node.transform().decomposed();
            child
                .components
                .push(Box::new(sample_data::offline::Transform {
                    position: Vec3::new(position[0], position[1], -position[2]),
                    rotation: lgn_math::Quat::from_xyzw(
                        rotation[0],
                        rotation[1],
                        -rotation[2],
                        -rotation[3],
                    ),
                    scale: scale.into(),
                }));
            if let Some(mesh) = node.mesh() {
                let visual = Box::new(lgn_graphics_data::offline::Visual {
                    renderable_geometry: Some(ResourcePathId::from(resource_id).push_named(
                        lgn_graphics_data::runtime::Model::TYPE,
                        mesh.name().unwrap(),
                    )),
                    color_blend: 0.0,
                    ..Visual::default()
                });
                child.components.push(visual);
            }

            if project.exists(child_id).await {
                project.save_resource(child_id, &child).await.unwrap();
            } else {
                project
                    .add_resource_with_id(child_id, &child)
                    .await
                    .unwrap();
            }
        }

        let mut gltf_loader = offline_data::GltfLoader::default();
        for mesh in gltf.document.meshes() {
            gltf_loader
                .models
                .push(ResourcePathId::from(resource_id).push_named(
                    lgn_graphics_data::runtime::Model::TYPE,
                    mesh.name().unwrap(),
                ));
        }

        let (materials, texture_references) =
            lgn_graphics_data::gltf_utils::extract_materials_from_document(
                &gltf.document,
                resource_id,
            );

        gltf_loader.textures.extend(texture_references);

        for (_material, material_name) in &materials {
            gltf_loader.materials.push(
                ResourcePathId::from(resource_id)
                    .push_named(lgn_graphics_data::runtime::Material::TYPE, material_name), //.push(lgn_graphics_data::runtime::Material::TYPE),
            );
        }
        root_entity.components.push(Box::new(gltf_loader));

        project.save_resource(root_id, &root_entity).await.unwrap();
    }

    Some(resource_id)
}
