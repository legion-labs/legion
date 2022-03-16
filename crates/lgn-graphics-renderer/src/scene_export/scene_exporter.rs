use lgn_app::{App, EventReader, Events, Plugin};
use lgn_ecs::prelude::{IntoExclusiveSystem, Local, Query, Res, ResMut, Without};
use lgn_transform::components::GlobalTransform;

use crate::{
    components::{
        CameraComponent, LightComponent, LightType, ManipulatorComponent, VisualComponent,
    },
    egui::egui_plugin::Egui,
    resources::{MeshManager, ModelManager},
};

type KhrLight = gltf::json::extensions::scene::khr_lights_punctual::Light;
type KhrLightType = gltf::json::extensions::scene::khr_lights_punctual::Type;

pub(crate) struct SceneExporterPlugin {}

impl Plugin for SceneExporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_scene_exporter);
        app.add_system(on_scene_export_requested.exclusive_system());
        app.add_event::<SceneExportRequested>();
    }
}

pub(crate) fn on_scene_export_requested(
    mut event_scene_export_requested: EventReader<'_, '_, SceneExportRequested>,
    visuals: Query<'_, '_, (&VisualComponent, &GlobalTransform), Without<ManipulatorComponent>>,
    lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    cameras: Query<'_, '_, (&CameraComponent, &GlobalTransform)>,
    model_manager: Res<ModelManager>,
    mesh_manager: Res<MeshManager>,
) {
    if event_scene_export_requested.is_empty() {
        return;
    }
    let dir = format!(
        "{}/scene_export",
        event_scene_export_requested
            .iter()
            .next()
            .unwrap()
            .export_path
    );
    let _ = std::fs::create_dir_all(&dir).unwrap();

    let mut nodes = Vec::new();
    let mut scene_node_indices = Vec::new();
    let mut node_idx = 0;

    let extensions = Some(export_extensions(
        &mut nodes,
        &lights,
        &mut node_idx,
        &mut scene_node_indices,
    ));

    let scene = gltf::json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Legion scene".into()),
        nodes: scene_node_indices,
    };

    for (visual, transform) in visuals.iter() {
        let (model_meta_data, ready) = model_manager.get_model_meta_data(visual);
        if !ready {
            todo!()
        }

        for mesh in &model_meta_data.meshes {
            let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
            mesh_meta_data.source_data.positions
        }
    }

    let buffer = gltf::json::Buffer {
        byte_length: todo!(),
        name: todo!(),
        uri: todo!(),
        extensions: todo!(),
        extras: todo!(),
    };

    let buffers = vec![];

    let root = gltf::json::Root {
        extensions,
        //accessors: todo!(),
        //animations: todo!(),
        //asset: todo!(),
        //buffers: todo!(),
        //buffer_views: todo!(),
        //scene: todo!(),
        //extras: todo!(),
        extensions_used: vec!["KHR_lights_punctual".into()],
        extensions_required: vec!["KHR_lights_punctual".into()],
        //cameras: todo!(),
        //images: todo!(),
        //materials: todo!(),
        //meshes: todo!(),
        nodes,
        //samplers: todo!(),
        scenes: vec![scene],
        //skins: todo!(),
        //textures: todo!(),
        ..gltf::json::Root::default()
    };
    let writer = std::fs::File::create(format!("{}/scene.gltf", dir)).expect("I/O error");
    gltf::json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");

    //let bin = to_padded_byte_vector(triangle_vertices);
    //let mut writer = fs::File::create("triangle/buffer0.bin").expect("I/O error");
    //writer.write_all(&bin).expect("I/O error");
}

fn export_extensions(
    nodes: &mut Vec<gltf::json::Node>,
    lights: &Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    node_idx: &mut u32,
    scene_node_indices: &mut Vec<gltf::json::Index<gltf::json::Node>>,
) -> gltf::json::extensions::root::Root {
    let mut light_idx = 0;
    let mut khr_lights = Vec::new();
    for (light, transform) in lights.iter() {
        khr_lights.push(export_light(light));
        let node = gltf::json::Node {
            camera: None,
            children: None,
            extensions: Some(gltf::json::extensions::scene::Node {
                khr_lights_punctual: Some(
                    gltf::json::extensions::scene::khr_lights_punctual::KhrLightsPunctual {
                        light: gltf::json::Index::new(light_idx),
                    },
                ),
            }),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some(format!("Light {} orientation", light_idx)),
            rotation: Some(gltf::json::scene::UnitQuaternion(transform.rotation.into())),
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        };
        nodes.push(node);
        let node = gltf::json::Node {
            camera: None,
            children: Some(vec![gltf::json::Index::new(*node_idx)]),
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some(format!("Light {}", light_idx)),
            rotation: Some(gltf::json::scene::UnitQuaternion(transform.rotation.into())),
            scale: None,
            translation: Some(transform.translation.into()),
            skin: None,
            weights: None,
        };
        nodes.push(node);
        scene_node_indices.push(gltf::json::Index::new(*node_idx + 1));
        light_idx += 1;
        *node_idx += 2;
    }
    let extensions = gltf::json::extensions::root::Root {
        khr_lights_punctual: Some(gltf::json::extensions::root::KhrLightsPunctual {
            lights: khr_lights,
        }),
    };
    extensions
}

fn export_light(light: &LightComponent) -> KhrLight {
    KhrLight {
        color: light.color.to_array(),
        extensions: None,
        extras: gltf::json::Extras::default(),
        intensity: light.radiance,
        name: None,
        range: None,
        spot: match light.light_type {
            LightType::Spotlight { cone_angle } => {
                Some(gltf::json::extensions::scene::khr_lights_punctual::Spot {
                    inner_cone_angle: cone_angle * 0.9,
                    outer_cone_angle: cone_angle,
                })
            }
            _ => None,
        },
        type_: gltf::json::validation::Checked::Valid(match light.light_type {
            LightType::Directional => KhrLightType::Directional,
            LightType::Omnidirectional => KhrLightType::Point,
            LightType::Spotlight { cone_angle } => KhrLightType::Spot,
        }),
    }
}

pub(crate) struct UiData {
    path: String,
}

impl Default for UiData {
    fn default() -> Self {
        UiData {
            path: String::from("C:/scene_export"),
        }
    }
}

pub(crate) fn ui_scene_exporter(
    egui: Res<'_, Egui>,
    mut event_scene_export_requested: ResMut<'_, Events<SceneExportRequested>>,
    mut ui_data: Local<'_, UiData>,
) {
    egui::Window::new("Scene export").show(&egui.ctx, |ui| {
        ui.text_edit_singleline(&mut ui_data.path);
        if ui.button("Export").clicked() {
            event_scene_export_requested.send(SceneExportRequested {
                export_path: ui_data.path.clone(),
            });
        }
    });
}

pub(crate) struct SceneExportRequested {
    export_path: String,
}
