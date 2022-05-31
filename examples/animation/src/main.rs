//! Utility to regenerate the data for the animation demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use clap::{ArgEnum, Parser};
use lgn_source_control::RepositoryName;
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_animation::offline::{
    AnimationTrack, AnimationTransform, AnimationTransformBundle, VecAnimationTransform,
};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourcePathName};
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistry, AssetRegistryOptions, Component, ResourceDescriptor,
    ResourceId, ResourcePathId, ResourceTypeAndId,
};
use lgn_data_transaction::BuildManager;
use lgn_graphics_data::offline::CameraSetup;
use lgn_graphics_renderer::components::Mesh;
use lgn_math::prelude::{Quat, Vec3};
use lgn_physics::offline::PhysicsSceneSettings;

use lgn_tracing::LevelFilter;
use sample_data::{
    offline::{Light, Transform, Visual},
    LightType,
};

#[derive(Debug, Copy, Clone, PartialEq, ArgEnum)]
enum CompilersSource {
    InProcess,
    External,
    Remote,
}

#[derive(Parser)]
#[clap(name = "Animation data re-builder")]
struct Args {
    /// Compile resources remotely.
    #[clap(arg_enum, default_value = "in-process")]
    compilers: CompilersSource,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuardBuilder::default()
        .with_local_sink_max_level(LevelFilter::Info)
        .build();

    let project_dir = PathBuf::from("examples/animation/data");

    clean_folders(&project_dir);

    let build_dir = project_dir.join("temp");
    std::fs::create_dir_all(&build_dir).unwrap();

    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "examples-animation".parse().unwrap();

    // Ensure the repository exists.
    let _index = repository_index
        .ensure_repository(repository_name.clone())
        .await;

    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    let absolute_project_dir = {
        if !project_dir.is_absolute() {
            std::env::current_dir().unwrap().join(&project_dir)
        } else {
            project_dir.clone()
        }
    };
    let mut project = Project::create(
        absolute_project_dir,
        &repository_index,
        repository_name,
        Arc::clone(&source_control_content_provider),
    )
    .await
    .expect("failed to create a project");

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Arc::clone(&data_content_provider), Manifest::default());
    lgn_graphics_data::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    sample_data::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create().await;

    let resource_ids = create_offline_data(&mut project, &asset_registry).await;
    project
        .commit("initial commit")
        .await
        .expect("failed to commit");

    let mut compilers_path = env::current_exe().expect("cannot access current_exe");
    compilers_path.pop(); // pop the .exe name

    let compilers = match args.compilers {
        CompilersSource::InProcess => CompilerRegistryOptions::default()
            .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
            .add_compiler(&lgn_compiler_runtime_model::COMPILER_INFO),
        CompilersSource::External => CompilerRegistryOptions::local_compilers(compilers_path),
        CompilersSource::Remote => lgn_data_compiler_remote::compiler_node::remote_compilers(
            compilers_path,
            "http://localhost:2020/",
        ),
    };

    let absolute_build_dir = {
        if !build_dir.is_absolute() {
            std::env::current_dir().unwrap().join(&build_dir)
        } else {
            build_dir.clone()
        }
    };
    let data_build = DataBuildOptions::new_with_sqlite_output(
        &absolute_build_dir,
        compilers,
        Arc::clone(&data_content_provider),
    )
    .asset_registry(asset_registry.clone());

    let mut build_manager = BuildManager::new(
        data_build,
        &project,
        Manifest::default(),
        Manifest::default(),
    )
    .await
    .unwrap();

    for id in resource_ids {
        let derived = build_manager.build_all_derived(id, &project).await.unwrap();
        println!("derived: {}, offline: {}", derived.0.resource_id(), id);
    }

    let runtime_dir = project_dir.join("runtime");
    std::fs::create_dir(&runtime_dir).unwrap();
    let runtime_manifest_path = runtime_dir.join("game.manifest");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(runtime_manifest_path)
        .expect("open file");
    write!(file, "{}", build_manager.get_manifest_id())
        .expect("failed to write manifest id to file");
    Ok(())
}

fn clean_folders(project_dir: impl AsRef<Path>) {
    let mut path = project_dir.as_ref().to_owned();

    let mut clean = |sub_path| {
        path.push(sub_path);
        if path.exists() {
            let remove = if path.is_dir() {
                std::fs::remove_dir_all
            } else {
                std::fs::remove_file
            };
            remove(path.as_path()).unwrap_or_else(|_| panic!("Cannot delete {:?}", path));
        }
        path.pop();
    };

    clean("remote");
    clean("offline");
    clean("runtime");
    clean("temp");
    clean("project.index");
}

async fn create_offline_data(
    project: &mut Project,
    resource_registry: &AssetRegistry,
) -> Vec<ResourceTypeAndId> {
    /* Animation skeleton */
    let cube_model_id = create_offline_model(
        project,
        resource_registry,
        "84510591-4ccd-4818-bf73-15375e6b140a",
        "/scene/models/skeleton.mod",
        Mesh::new_cube(0.1),
    )
    .await;

    let light_id = create_offline_entity(
        project,
        resource_registry,
        "85701c5f-f9f8-4ca0-9111-8243c4ea2cd6",
        "/scene/light.ent",
        vec![
            Box::new(Transform {
                position: (0_f32, 10_f32, 2_f32).into(),
                ..Transform::default()
            }),
            Box::new(Light {
                light_type: LightType::Directional,
                color: (0xFF, 0xFF, 0xEF).into(),
                radiance: 12_f32,
                enabled: true,
                ..Light::default()
            }),
        ],
        vec![],
    )
    .await;

    let skeleton = create_offline_entity(
        project,
        resource_registry,
        "825f5f93-7a86-4616-81aa-ce3e146248ab",
        "/scene/skeleton.ent",
        vec![
            Box::new(Transform {
                position: (-0.5_f32, 0.0_f32, 0.0_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0x20, 0xFF, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(AnimationTrack {
                key_frames: vec![
                    VecAnimationTransform {
                        anim_transform_vec: vec![
                            /* Bone 1 root */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                            },
                            /* Head */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.7, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                            },
                            /* Bone 2 shoulder */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.4, 0.3, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                            },
                            /* Bone 3 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.15, -0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 4 arm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.15, -0.15, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 5 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.15, -0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 6 elbow */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 7 forearm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 8 forearm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.2, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 9 hip */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 10 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 11 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 12 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 13 hip2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 14 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 15 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 16 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                        ],
                    },
                    VecAnimationTransform {
                        anim_transform_vec: vec![
                            /* Bone 1 root */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                            },
                            /* Head */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.7, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                            },
                            /* Bone 2 shoulder */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.4, 0.3, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                            },
                            /* Bone 3 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, -0.1, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 4 arm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.2, -0.15, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 5 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, -0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 6 elbow */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 7 forearm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.15, 0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 8 forearm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.15, 0.15, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 9 hip */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 10 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 11 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 12 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 13 hip2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 14 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 15 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 16 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                        ],
                    },
                    VecAnimationTransform {
                        anim_transform_vec: vec![
                            /* Bone 1 root */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.4),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.4),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                            },
                            /* Head */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.7, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                            },
                            /* Bone 2 shoulder */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.4, 0.3, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                            },
                            /* Bone 3 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.3, -0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 4 arm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.2, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 5 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 6 elbow */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 7 forearm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.2, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 8 forearm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.2, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 9 hip */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 10 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 11 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 12 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 13 hip2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 14 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 15 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 16 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                        ],
                    },
                    VecAnimationTransform {
                        anim_transform_vec: vec![
                            /* Bone 1 root */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.4, 0.0, 0.2),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(2.0, 2.0, 2.0),
                                },
                            },
                            /* Head */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.7, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(1.0, 1.0, 1.0),
                                },
                            },
                            /* Bone 2 shoulder */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.4, 0.3, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.8, 0.8, 0.8),
                                },
                            },
                            /* Bone 3 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, -0.1, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 4 arm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.2, -0.15, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 5 arm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, -0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 6 elbow */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.2, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            /* Bone 7 forearm */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.15, 0.15, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.5, 0.5, 0.5),
                                },
                            },
                            // /* Bone 8 forearm */
                            // AnimationTransformBundle {
                            //     local: AnimationTransform {
                            //         translation: Vec3::new(-0.15, 0.15, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            //     global: AnimationTransform {
                            //         translation: Vec3::new(0.0, 0.0, 0.0),
                            //         rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                            //         scale: Vec3::new(0.5, 0.5, 0.5),
                            //     },
                            // },
                            /* Bone 9 hip */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 10 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 11 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 12 leg */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(-0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 13 hip2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.35, -0.4, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 14 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 15 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                            /* Bone 16 leg2 */
                            AnimationTransformBundle {
                                local: AnimationTransform {
                                    translation: Vec3::new(0.05, -0.2, -0.1),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                                global: AnimationTransform {
                                    translation: Vec3::new(0.0, 0.0, 0.0),
                                    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
                                    scale: Vec3::new(0.6, 0.6, 0.6),
                                },
                            },
                        ],
                    },
                ],
                current_key_frame_index: 0,
                duration_key_frames: vec![1.0, 1.0, 1.0, 1.0, 1.0],
                time_since_last_tick: 0.0,
                looping: true,
                bone_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
                parent_indices: vec![-1, 0, 0, 2, 3, 4, 5, 0, 7, 8, 9, 0, 11, 12, 13], // Stores the bone idx of every bones parent, if -1: root bone
            }),
        ],
        vec![],
    )
    .await;

    let scene_id = create_offline_entity(
        project,
        resource_registry,
        "09f7380d-51b2-4061-9fe4-52ceccce55e7",
        "/scene.ent",
        vec![
            Box::new(CameraSetup {
                eye: Vec3::new(0.0, 1.2, -3.0),
                look_at: Vec3::new(0.0, 0.5, 0.0),
            }),
            Box::new(PhysicsSceneSettings {
                gravity: Vec3::new(0.0, -1.0, 0.0),
            }),
        ],
        vec![light_id, skeleton],
    )
    .await;

    vec![scene_id.source_resource()]
}

async fn create_offline_entity(
    project: &mut Project,
    resources: &AssetRegistry,
    resource_id: &str,
    resource_path: &str,
    components: Vec<Box<dyn Component>>,
    children: Vec<ResourcePathId>,
) -> ResourcePathId {
    let kind = sample_data::offline::Entity::TYPE;
    let id = resource_id
        .parse::<ResourceId>()
        .expect("invalid ResourceId format");
    let type_id = ResourceTypeAndId { kind, id };
    let name: ResourcePathName = resource_path.into();

    let exists = project.exists(id).await;
    let handle = if exists {
        project
            .load_resource(type_id, resources)
            .expect("failed to load resource")
    } else {
        resources
            .new_resource_with_id(type_id)
            .expect("failed to create new resource")
    };

    let mut entity = handle
        .instantiate::<sample_data::offline::Entity>(resources)
        .unwrap();
    entity.components.clear();
    entity.components.extend(components.into_iter());
    entity.children.clear();
    entity.children.extend(children.into_iter());

    handle.apply(entity, resources);

    if exists {
        project
            .save_resource(type_id, handle, resources)
            .await
            .expect("failed to save resource");
    } else {
        project
            .add_resource_with_id(
                name,
                sample_data::offline::Entity::TYPENAME,
                kind,
                id,
                handle,
                resources,
            )
            .await
            .expect("failed to add new resource");
    }

    let path: ResourcePathId = type_id.into();
    path.push(sample_data::runtime::Entity::TYPE)
}

async fn create_offline_model(
    project: &mut Project,
    resources: &AssetRegistry,
    resource_id: &str,
    resource_path: &str,
    mesh: Mesh,
) -> ResourcePathId {
    let kind = lgn_graphics_data::offline::Model::TYPE;
    let id = resource_id
        .parse::<ResourceId>()
        .expect("invalid ResourceId format");
    let type_id = ResourceTypeAndId { kind, id };
    let name: ResourcePathName = resource_path.into();

    let exists = project.exists(id).await;
    let handle = if exists {
        project
            .load_resource(type_id, resources)
            .expect("failed to load resource")
    } else {
        resources
            .new_resource_with_id(type_id)
            .expect("failed to create new resource")
    };

    let mut model = handle
        .instantiate::<lgn_graphics_data::offline::Model>(resources)
        .unwrap();
    model.meshes.clear();

    let mesh = lgn_graphics_data::offline::Mesh {
        positions: mesh.positions,
        normals: mesh.normals.unwrap(),
        tangents: mesh.tangents.unwrap(),
        tex_coords: mesh.tex_coords.unwrap(),
        indices: mesh.indices.unwrap(),
        colors: mesh
            .colors
            .map(|colors| colors.iter().map(|color| (*color).into()).collect())
            .unwrap(),
        material: None,
    };
    model.meshes.push(mesh);

    handle.apply(model, resources);

    if exists {
        project
            .save_resource(type_id, handle, resources)
            .await
            .expect("failed to save resource");
    } else {
        project
            .add_resource_with_id(
                name,
                lgn_graphics_data::offline::Model::TYPENAME,
                kind,
                id,
                handle,
                resources,
            )
            .await
            .expect("failed to add new resource");
    }

    let path: ResourcePathId = type_id.into();
    path.push(lgn_graphics_data::runtime::Model::TYPE)
}

// let body_model_id = create_offline_model(
//     project,
//     resource_registry,
//     "c26b19b2-e80a-4db5-a53c-ba24492d8015",
//     "/scene/models/body.mod",
//     Mesh::new_cylinder(0.25, 1.0, 60),
// )
// .await;

// let arm_model_id = create_offline_model(
//     project,
//     resource_registry,
//     "11b56c2e-a3d7-4ecc-99c8-2b0153c5feb0",
//     "/scene/models/arm.mod",
//     Mesh::new_cylinder(0.1, 0.5, 60),
// )
// .await;

// let cube_model_id = create_offline_model(
//     project,
//     resource_registry,
//     "c26b19b2-e80a-4db5-a53c-ba24492d8015",
//     "/scene/models/body.mod",
//     Mesh::new_cube(0.5),
// )
// .await;

// let sphere_model_id = create_offline_model(
//     project,
//     resource_registry,
//     "53db2f32-ce66-4e01-b06f-960aaa7712e4",
//     "/scene/models/sphere.mod",
//     Mesh::new_sphere(0.25, 64, 64),
// )
// .await;

// let ball_a_id = create_offline_entity(
//     project,
//     resource_registry,
//     "4dd86281-c3d0-4040-aa59-b3c6cc84eb83",
//     "/scene/ball-a.ent",
//     vec![
//         Box::new(Transform {
//             position: (0.2_f32, 1.25_f32, 0.5_f32).into(),
//             ..Transform::default()
//         }),
//         Box::new(Visual {
//             renderable_geometry: Some(sphere_model_id.clone()),
//             color: (0x20, 0x4F, 0xFF).into(),
//             ..Visual::default()
//         }),
//     ],
//     vec![],
// )
// .await;

// let arm = create_offline_entity(
//     project,
//     resource_registry,
//     "d6681749-9d52-4092-b49c-676a8da53540",
//     "/scene/arm.ent",
//     vec![
//         Box::new(Transform {
//             position: (0.55_f32, 0.5_f32, 0.5_f32).into(),
//             ..Transform::default()
//         }),
//         Box::new(Visual {
//             renderable_geometry: Some(arm_model_id.clone()),
//             color: (0xFF, 0x4F, 0xFF).into(),
//             ..Visual::default()
//         }),
//         Box::new(AnimationTrack {
//             key_frames: vec![
//                 Vec3::new(0.5, -0.5, -0.5),
//                 Vec3::new(0.5, -0.5, 0.5),
//                 Vec3::new(-0.5, -0.5, -0.5),
//                 Vec3::new(-0.5, -0.5, 0.5),
//                 Vec3::new(0.0, 0.0, 0.0),
//             ],
//             current_key_frame_index: 0,
//             duration_key_frames: vec![1.0, 1.0, 1.0, 1.0, 1.0],
//             time_since_last_tick: 0.0,
//             looping: true,
//         }),
//     ],
//     vec![],
// )
// .await;

// let cube_2 = create_offline_entity(
//     project,
//     resource_registry,
//     "5882f9f5-6ed2-4d25-8cf3-bfcea61134e8",
//     "/scene/cube_2.ent",
//     vec![
//         Box::new(Transform {
//             position: (-0.2_f32, 0.0_f32, -0.5_f32).into(),
//             ..Transform::default()
//         }),
//         Box::new(Visual {
//             renderable_geometry: Some(cube_model_id.clone()),
//             color: (0x2F, 0xFF, 0xFF).into(),
//             ..Visual::default()
//         }),
//         Box::new(AnimationTrack {
//             key_frames: vec![
//                 Vec3::new(0.2, 0.0, 0.0),
//                 Vec3::new(0.4, -0.0, 0.0),
//                 Vec3::new(0.2, -0.0, -0.0),
//                 Vec3::new(0.0, -0.0, 0.0),
//                 Vec3::new(-0.2, 0.0, 0.0),
//                 Vec3::new(0.0, 0.0, 0.0),
//             ],
//             current_key_frame_index: 0,
//             duration_key_frames: vec![1.0, 1.0, 1.0, 1.0, 1.0],
//             time_since_last_tick: 0.0,
//             looping: true,
//             bone_ids: vec![0, 1],
//             parent_indices: vec![-1, 0], // Stores the bone idx of every bones parent, if -1: root bone
//             local_reference_pose: vec![AnimationTransform {
//                 translation: vec![0.0, 0.0, 0.0, 0.0],
//                 rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
//                 scale: vec![0.0, 0.0, 0.0, 0.0],
//             }],
//             global_reference_pose: vec![AnimationTransform {
//                 translation: vec![0.0, 0.0, 0.0, 0.0],
//                 rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
//                 scale: vec![0.0, 0.0, 0.0, 0.0],
//             }],
//         }),
//     ],
//     vec![],
// )
// .await;

// let animation_data = create_offline_entity(
//     project,
//     resource_registry,
//     "12e04cd1-5c7f-4ade-9565-56e554268b9f",
//     "/scene/animation.ent",
//     vec![Box::new(AnimationTrack {
//         key_frames: vec![
//             Vec3::new(0.2, 0.0, 0.0),
//             Vec3::new(0.4, -0.0, 0.0),
//             Vec3::new(0.2, -0.0, -0.0),
//             Vec3::new(0.0, -0.0, 0.0),
//             Vec3::new(-0.2, 0.0, 0.0),
//             Vec3::new(0.0, 0.0, 0.0),
//         ],
//         current_key_frame_index: 0,
//         duration_key_frames: vec![1.0, 1.0, 1.0, 1.0, 1.0],
//         time_since_last_tick: 0.0,
//         looping: true,
//         bone_ids: vec![0, 1],
//         parent_indices: vec![-1, 0], // Stores the bone idx of every bones parent, if -1: root bone
//         local_reference_pose: vec![AnimationTransform {
//             translation: vec![0.0, 0.0, 0.0, 0.0],
//             rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
//             scale: vec![0.0, 0.0, 0.0, 0.0],
//         }],
//         global_reference_pose: vec![AnimationTransform {
//             translation: vec![0.0, 0.0, 0.0, 0.0],
//             rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
//             scale: vec![0.0, 0.0, 0.0, 0.0],
//         }],
//     })],
//     vec![],
// )
// .await;
