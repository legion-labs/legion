//! Utility to regenerate the data for the physics demo.
//! All the data is created from scratch with this source code, i.e. no corresponding "raw" data

// crate-specific lint exceptions:
//#![allow()]

use std::{
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
    manifest::Manifest, AssetRegistryOptions, Resource, ResourceId, ResourceTypeAndId,
};
use lgn_data_transaction::BuildManager;
use lgn_math::prelude::*;
use lgn_physics::{
    offline::{PhysicsRigidBox, PhysicsRigidConvexMesh, PhysicsRigidSphere},
    RigidActorType,
};
use lgn_renderer::components::Mesh;
use sample_data::offline::{Transform, Visual};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default().unwrap();

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

    let compilers = CompilerRegistryOptions::default()
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_runtime_model::COMPILER_INFO);

    let data_build = DataBuildOptions::new(&build_dir, compilers)
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

async fn create_offline_model(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
    resource_id: &str,
    mesh: Mesh,
    resource_path: &str,
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

async fn create_offline_data(
    project: &mut Project,
    resource_registry: &Arc<Mutex<ResourceRegistry>>,
) -> Vec<ResourceTypeAndId> {
    let cube_model_id = create_offline_model(
        project,
        resource_registry,
        "c26b19b2-e80a-4db5-a53c-ba24492d8015",
        Mesh::new_cube(0.5),
        "/scene/models/cube.mod",
    )
    .await;

    let sphere_model_id = create_offline_model(
        project,
        resource_registry,
        "53db2f32-ce66-4e01-b06f-960aaa7712e4",
        Mesh::new_sphere(0.25, 64, 64),
        "/scene/models/sphere.mod",
    )
    .await;

    let ground_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("8859bf63-f187-4aa3-afd5-425130c7ba04").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (0_f32, -1_f32, 0_f32).into(),
            rotation: Quat::from_rotation_x(-0.01_f32),
            scale: (12_f32, 0.1_f32, 12_f32).into(),
        }));
        entity.components.push(Box::new(Visual {
            renderable_geometry: Some(cube_model_id.clone()),
            color: (0x10, 0x10, 0x55).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidBox {
            actor_type: RigidActorType::Static,
            half_extents: (3_f32, 0.25_f32, 3_f32).into(),
        }));

        project
            .add_resource_with_id(
                "/scene/ground.ent".into(),
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
    };

    let box_a_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("8e04418d-ca9a-4e4a-a0ea-68e74d8c10d0").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (-1_f32, 0.1_f32, 1_f32).into(),
            rotation: Quat::from_rotation_z(0.3_f32),
            ..Transform::default()
        }));
        entity.components.push(Box::new(Visual {
            renderable_geometry: Some(cube_model_id.clone()),
            color: (0xFF, 0xFF, 0x20).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidBox {
            actor_type: RigidActorType::Dynamic,
            half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
        }));

        project
            .add_resource_with_id(
                "/scene/box-a.ent".into(),
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
    };

    let box_b_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("b355fa97-97ee-44f9-afbf-1c2920ce5064").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (-1_f32, 1.3_f32, 1.2_f32).into(),
            rotation: Quat::from_rotation_x(-0.2_f32),
            ..Transform::default()
        }));
        entity.components.push(Box::new(Visual {
            renderable_geometry: Some(cube_model_id.clone()),
            color: (0xFF, 0x20, 0xFF).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidBox {
            actor_type: RigidActorType::Dynamic,
            half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
        }));

        project
            .add_resource_with_id(
                "/scene/box-b.ent".into(),
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
    };

    let box_c_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("6715d493-d7d4-4155-bc18-0e1795c53580").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (-0.5_f32, 0_f32, 0.5_f32).into(),
            rotation: Quat::from_rotation_y(1_f32),
            ..Transform::default()
        }));
        entity.components.push(Box::new(Visual {
            renderable_geometry: Some(cube_model_id.clone()),
            color: (0x20, 0xFF, 0xFF).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidBox {
            actor_type: RigidActorType::Dynamic,
            half_extents: (0.25_f32, 0.25_f32, 0.25_f32).into(),
        }));

        project
            .add_resource_with_id(
                "/scene/box-c.ent".into(),
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
    };

    let ball_a_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("4dd86281-c3d0-4040-aa59-b3c6cc84eb83").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (1.2_f32, 3.3_f32, 0.5_f32).into(),
            ..Transform::default()
        }));
        entity.components.push(Box::new(Visual {
            renderable_geometry: Some(sphere_model_id.clone()),
            color: (0x20, 0x4F, 0xFF).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidSphere {
            actor_type: RigidActorType::Dynamic,
            radius: 0.25_f32,
        }));

        project
            .add_resource_with_id(
                "/scene/ball-a.ent".into(),
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
    };

    let pyramid_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("9ffa97ef-3ae1-4859-aaeb-91f7268cad50").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();
        entity.components.push(Box::new(Transform {
            position: (0_f32, 0_f32, 0_f32).into(),
            ..Transform::default()
        }));
        entity.components.push(Box::new(Visual {
            color: (0x00, 0x00, 0x00).into(),
            ..Visual::default()
        }));
        entity.components.push(Box::new(PhysicsRigidConvexMesh {
            actor_type: RigidActorType::Dynamic,
            vertices: vec![Vec3::Y, Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z],
            ..PhysicsRigidConvexMesh::default()
        }));

        project
            .add_resource_with_id(
                "/scene/pyramid.ent".into(),
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
    };

    let scene_id = {
        let mut resources = resource_registry.lock().await;
        let id = ResourceTypeAndId {
            kind: sample_data::offline::Entity::TYPE,
            id: ResourceId::from_str("09f7380d-51b2-4061-9fe4-52ceccce55e7").unwrap(),
        };
        let handle = resources.new_resource(id.kind).unwrap();

        let entity = handle
            .get_mut::<sample_data::offline::Entity>(&mut resources)
            .unwrap();

        entity.children.push(box_a_id);
        entity.children.push(box_b_id);
        entity.children.push(box_c_id);
        entity.children.push(ball_a_id);
        entity.children.push(pyramid_id);
        entity.children.push(ground_id);

        project
            .add_resource_with_id(
                "/scene.ent".into(),
                sample_data::offline::Entity::TYPENAME,
                id.kind,
                id.id,
                handle,
                &mut resources,
            )
            .await
            .unwrap();
        id
    };

    vec![scene_id]
}
