//! Utility to regenerate the data for the physics demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use clap::{ArgEnum, Parser};
use lgn_source_control::RepositoryName;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf, sync::Arc};

use clap::{ArgEnum, Parser};
use lgn_content_store::indexing::{empty_tree_id, SharedTreeIdentifier};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::{Project, ResourcePathName};
use lgn_data_runtime::{
    AssetRegistry, AssetRegistryOptions, Component, ResourceDescriptor, ResourceId, ResourcePathId,
    ResourceTypeAndId,
};
use lgn_data_transaction::BuildManager;
use lgn_graphics_data::offline::CameraSetup;
use lgn_graphics_renderer::components::Mesh;
use lgn_math::prelude::{Quat, Vec3};
use lgn_physics::{
    offline::{PhysicsRigidBox, PhysicsRigidConvexMesh, PhysicsRigidSphere, PhysicsSceneSettings},
    RigidActorType,
};
use lgn_source_control::RepositoryName;
use lgn_tracing::{info, LevelFilter};
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
#[clap(name = "Physics data rebuilder")]
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

    let project_dir = PathBuf::from("examples/physics/data");
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir)
            .unwrap_or_else(|_| panic!("Cannot delete {}", project_dir.display()));
    }
    std::fs::create_dir_all(&project_dir).unwrap();

    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "examples-physics".parse().unwrap();

    // Ensure the repository exists.
    let _index = repository_index.ensure_repository(&repository_name).await;

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
        &repository_name,
        Arc::clone(&source_control_content_provider),
    )
    .await
    .expect("failed to create a project");

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_cas_with_empty_manifest(Arc::clone(&data_content_provider))
        .await
        .add_device_persistent_cs(Arc::clone(&source_control_content_provider), || {
            (&project).get_manifest_id().clone()
        });
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

    let build_dir = project_dir.join("temp");
    std::fs::create_dir_all(&build_dir).unwrap();
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

    let mut build_manager = BuildManager::new(&project, data_build, None).await.unwrap();

    for id in resource_ids {
        let derived_result = build_manager.build_all_derived(id, &project).await.unwrap();
        info!("{} -> {}", id, derived_result.0.resource_id());
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
    write!(file, "{}", runtime_manifest_id.read()).expect("failed to write manifest id to file");
    Ok(())
}

async fn create_offline_data(
    project: &mut Project,
    resource_registry: &AssetRegistry,
) -> Vec<ResourceTypeAndId> {
    let cube_model_id = create_offline_model(
        project,
        resource_registry,
        "c26b19b2-e80a-4db5-a53c-ba24492d8015",
        "/scene/models/cube.mod",
        Mesh::new_cube(0.5),
    )
    .await;

    let sphere_model_id = create_offline_model(
        project,
        resource_registry,
        "53db2f32-ce66-4e01-b06f-960aaa7712e4",
        "/scene/models/sphere.mod",
        Mesh::new_sphere(0.25, 64, 64),
    )
    .await;

    let pyramid_model_id = create_offline_model(
        project,
        resource_registry,
        "5e0d46c5-78da-4c5e-8204-a2c859ec5c09",
        "/scene/models/pyramid.mod",
        Mesh::new_pyramid(1.0, 0.5),
    )
    .await;

    let ground_id = create_offline_entity(
        project,
        resource_registry,
        "8859bf63-f187-4aa3-afd5-425130c7ba04",
        "/scene/ground.ent",
        vec![
            Box::new(Transform {
                position: (0_f32, -1_f32, 0_f32).into(),
                rotation: Quat::from_rotation_x(-0.01_f32),
                scale: (12_f32, 0.1_f32, 12_f32).into(),
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0x10, 0x10, 0x55).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidBox {
                actor_type: RigidActorType::Static,
                half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
            }),
        ],
        vec![],
    )
    .await;

    let box_a_id = create_offline_entity(
        project,
        resource_registry,
        "8e04418d-ca9a-4e4a-a0ea-68e74d8c10d0",
        "/scene/box-a.ent",
        vec![
            Box::new(Transform {
                position: (-1_f32, 0.1_f32, 1_f32).into(),
                rotation: Quat::from_rotation_z(0.3_f32),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0xFF, 0xFF, 0x20).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidBox {
                actor_type: RigidActorType::Dynamic,
                half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
            }),
        ],
        vec![],
    )
    .await;

    let box_b_id = create_offline_entity(
        project,
        resource_registry,
        "b355fa97-97ee-44f9-afbf-1c2920ce5064",
        "/scene/box-b.ent",
        vec![
            Box::new(Transform {
                position: (-1_f32, 1.3_f32, 1.2_f32).into(),
                rotation: Quat::from_rotation_x(-0.2_f32),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0xFF, 0x20, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidBox {
                actor_type: RigidActorType::Dynamic,
                half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
            }),
        ],
        vec![],
    )
    .await;

    let box_c_id = create_offline_entity(
        project,
        resource_registry,
        "6715d493-d7d4-4155-bc18-0e1795c53580",
        "/scene/box-c.ent",
        vec![
            Box::new(Transform {
                position: (-0.5_f32, 0_f32, 0.5_f32).into(),
                rotation: Quat::from_rotation_y(1_f32),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(cube_model_id.clone()),
                color: (0x20, 0xFF, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidBox {
                actor_type: RigidActorType::Dynamic,
                half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
            }),
        ],
        vec![],
    )
    .await;

    let ball_a_id = create_offline_entity(
        project,
        resource_registry,
        "4dd86281-c3d0-4040-aa59-b3c6cc84eb83",
        "/scene/ball-a.ent",
        vec![
            Box::new(Transform {
                position: (0.2_f32, 2.3_f32, 0.5_f32).into(),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(sphere_model_id.clone()),
                color: (0x20, 0x4F, 0xFF).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidSphere {
                actor_type: RigidActorType::Dynamic,
                radius: 0.25_f32,
            }),
        ],
        vec![],
    )
    .await;

    let pyramid_id = create_offline_entity(
        project,
        resource_registry,
        "9ffa97ef-3ae1-4859-aaeb-91f7268cad50",
        "/scene/pyramid.ent",
        vec![
            Box::new(Transform {
                position: (0_f32, 0.6_f32, 0_f32).into(),
                rotation: Quat::from_rotation_z(1_f32),
                ..Transform::default()
            }),
            Box::new(Visual {
                renderable_geometry: Some(pyramid_model_id.clone()),
                color: (0x00, 0x00, 0x00).into(),
                ..Visual::default()
            }),
            Box::new(PhysicsRigidConvexMesh {
                actor_type: RigidActorType::Dynamic,
                vertices: vec![
                    Vec3::new(0.5, -0.5, -0.5),
                    Vec3::new(0.5, -0.5, 0.5),
                    Vec3::new(-0.5, -0.5, -0.5),
                    Vec3::new(-0.5, -0.5, 0.5),
                    Vec3::new(0.0, 0.0, 0.0),
                ],
                ..PhysicsRigidConvexMesh::default()
            }),
        ],
        vec![],
    )
    .await;

    let light_id = create_offline_entity(
        project,
        resource_registry,
        "85701c5f-f9f8-4ca0-9111-8243c4ea2cd6",
        "/scene/light.ent",
        vec![
            Box::new(Transform {
                position: (0_f32, 10_f32, 0_f32).into(),
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

    let scene_id = create_offline_entity(
        project,
        resource_registry,
        "09f7380d-51b2-4061-9fe4-52ceccce55e7",
        "/scene.ent",
        vec![
            Box::new(CameraSetup {
                eye: Vec3::new(0.0, 1.2, -3.0),
                look_at: Vec3::ZERO,
            }),
            Box::new(PhysicsSceneSettings {
                gravity: Vec3::new(0.0, -1.0, 0.0),
            }),
        ],
        vec![
            box_a_id, box_b_id, box_c_id, ball_a_id, pyramid_id, ground_id, light_id,
        ],
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
            .await
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
            .await
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
        indices: mesh.indices,
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
