//! Utility to regenerate the data for the physics demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use clap::{ArgEnum, Parser};
use std::{
    env,
    fs::OpenOptions,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::{
    resource::{Project, ResourceRegistry, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::{
    manifest::Manifest, AssetRegistryOptions, Component, Resource, ResourceId, ResourceTypeAndId,
};
use lgn_data_transaction::BuildManager;
use lgn_graphics_renderer::components::Mesh;
use lgn_math::prelude::*;
use lgn_physics::{
    offline::{PhysicsRigidBox, PhysicsRigidConvexMesh, PhysicsRigidSphere},
    RigidActorType,
};
use lgn_tracing::LevelFilter;
use sample_data::{
    offline::{Light, Transform, Visual},
    LightType,
};
use tokio::sync::Mutex;

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
async fn main() {
    let args = Args::parse();

    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default()
        .unwrap()
        .with_log_level(LevelFilter::Info);

    let project_dir = PathBuf::from("examples/physics/data");

    clean_folders(&project_dir);

    let build_dir = project_dir.join("temp");
    std::fs::create_dir_all(&build_dir).unwrap();

    let absolute_project_dir = {
        if !project_dir.is_absolute() {
            std::env::current_dir().unwrap().join(&project_dir)
        } else {
            project_dir.clone()
        }
    };
    let mut project = Project::create_with_remote_mock(absolute_project_dir)
        .await
        .expect("failed to create a project");

    let mut resource_registry = ResourceRegistryOptions::new();
    lgn_graphics_data::offline::register_resource_types(&mut resource_registry);
    generic_data::offline::register_resource_types(&mut resource_registry);
    sample_data::offline::register_resource_types(&mut resource_registry);
    let content_store = HddContentStore::open(ContentStoreAddr::from(build_dir.clone())).unwrap();
    let resource_registry = resource_registry.create_async_registry();

    let resource_ids = create_offline_data(&mut project, &resource_registry).await;

    let mut asset_registry = AssetRegistryOptions::new()
        .add_device_dir(project.resource_dir())
        .add_device_cas(Box::new(content_store), Manifest::default());
    lgn_graphics_data::offline::add_loaders(&mut asset_registry);
    generic_data::offline::add_loaders(&mut asset_registry);
    sample_data::offline::add_loaders(&mut asset_registry);
    let asset_registry = asset_registry.create();

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

    let data_build = DataBuildOptions::new_with_sqlite_output(&build_dir, compilers)
        .content_store(&ContentStoreAddr::from(build_dir.as_path()))
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
        build_manager.build_all_derived(id, &project).await.unwrap();
    }

    let runtime_dir = project_dir.join("runtime");
    std::fs::create_dir(&runtime_dir).unwrap();
    let runtime_manifest_path = runtime_dir.join("game.manifest");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(runtime_manifest_path)
        .expect("open file");

    serde_json::to_writer_pretty(file, &build_manager.get_manifest()).expect("to write manifest");
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
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
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
        vec![],
        vec![
            box_a_id, box_b_id, box_c_id, ball_a_id, pyramid_id, ground_id, light_id,
        ],
    )
    .await;

    vec![scene_id.source_resource()]
}

async fn create_offline_entity(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    resource_id: &str,
    resource_path: &str,
    components: Vec<Box<dyn Component>>,
    children: Vec<ResourcePathId>,
) -> ResourcePathId {
    let mut resources = resource_registry.lock().await;
    let id = ResourceTypeAndId {
        kind: sample_data::offline::Entity::TYPE,
        id: ResourceId::from_str(resource_id).unwrap(),
    };
    let handle = resources.new_resource(id.kind).unwrap();

    let entity = handle
        .get_mut::<sample_data::offline::Entity>(&mut resources)
        .unwrap();
    entity.components.extend(components.into_iter());
    entity.children.extend(children.into_iter());

    project
        .add_resource_with_id(
            resource_path.into(),
            sample_data::offline::Entity::TYPENAME,
            id.kind,
            id.id,
            handle,
            &mut resources,
        )
        .await
        .unwrap();
    let path: ResourcePathId = id.into();
    path.push(sample_data::runtime::Entity::TYPE)
}

async fn create_offline_model(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    resource_id: &str,
    resource_path: &str,
    mesh: Mesh,
) -> ResourcePathId {
    let mut resources = resource_registry.lock().await;
    let id = ResourceTypeAndId {
        kind: lgn_graphics_data::offline::Model::TYPE,
        id: ResourceId::from_str(resource_id).unwrap(),
    };
    let handle = resources.new_resource(id.kind).unwrap();

    let model = handle
        .get_mut::<lgn_graphics_data::offline::Model>(&mut resources)
        .unwrap();
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

    project
        .add_resource_with_id(
            resource_path.into(),
            lgn_graphics_data::offline::Model::TYPENAME,
            id.kind,
            id.id,
            handle,
            &mut resources,
        )
        .await
        .unwrap();
    let path: ResourcePathId = id.into();
    path.push(lgn_graphics_data::runtime::Model::TYPE)
}
