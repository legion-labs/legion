#![allow(unsafe_code)]

use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use gltf::{
    json::{validation::Checked, Index, Node},
    Semantic,
};
use lgn_app::{App, EventReader, Events, Plugin};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{IntoExclusiveSystem, Local, Query, Res, ResMut, Without};
use lgn_math::{EulerRot, Vec3};
use lgn_tracing::error;
use lgn_transform::components::GlobalTransform;

use crate::{
    components::{
        CameraComponent, LightComponent, LightType, ManipulatorComponent, RenderSurface,
        VisualComponent,
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

struct SceneExportContext {
    output_dir: String,
    nodes: Vec<Node>,
    scene_nodes: Vec<Index<Node>>,
    buffers: Vec<gltf::json::Buffer>,
    buffer_views: Vec<gltf::json::buffer::View>,
    accessors: Vec<gltf::json::Accessor>,
    meshes: Vec<gltf::json::Mesh>,
}

impl SceneExportContext {
    fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
            nodes: Vec::new(),
            scene_nodes: Vec::new(),
            buffers: Vec::new(),
            buffer_views: Vec::new(),
            accessors: Vec::new(),
            meshes: Vec::new(),
        }
    }

    fn export_extensions(
        &mut self,
        lights: &Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    ) -> gltf::json::extensions::root::Root {
        let mut khr_lights = Vec::new();
        for (light, transform) in lights.iter() {
            let light_idx = khr_lights.len() as u32;
            khr_lights.push(Self::export_light(light));
            let node = gltf::json::Node {
                camera: None,
                children: None,
                extensions: Some(gltf::json::extensions::scene::Node {
                    khr_lights_punctual: Some(
                        gltf::json::extensions::scene::khr_lights_punctual::KhrLightsPunctual {
                            light: Index::new(light_idx),
                        },
                    ),
                }),
                extras: gltf::json::extras::Void::default(),
                matrix: None,
                mesh: None,
                name: Some(format!("Light {} orientation", light_idx)),
                // TODO: check transforms
                rotation: Some(gltf::json::scene::UnitQuaternion([
                    -transform.rotation.x,
                    -transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ])),
                scale: None,
                translation: Some([
                    transform.translation.x,
                    transform.translation.y,
                    -transform.translation.z,
                ]),
                skin: None,
                weights: None,
            };
            let node_id = self.nodes.len() as u32;
            self.scene_nodes.push(Index::new(node_id));
            self.nodes.push(node);
        }
        gltf::json::extensions::root::Root {
            khr_lights_punctual: Some(gltf::json::extensions::root::KhrLightsPunctual {
                lights: khr_lights,
            }),
        }
    }

    fn export_light(light: &LightComponent) -> KhrLight {
        KhrLight {
            color: light.color.as_linear().truncate().into(),
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
                LightType::Spotlight { cone_angle: _ } => KhrLightType::Spot,
            }),
        }
    }

    fn export_models(
        &mut self,
        visuals: &Query<
            '_,
            '_,
            (&VisualComponent, &GlobalTransform),
            Without<ManipulatorComponent>,
        >,
        model_manager: &Res<'_, ModelManager>,
        mesh_manager: &Res<'_, MeshManager>,
    ) -> Vec<ResourceTypeAndId> {
        let mut model_ids = HashSet::new();
        for (visual, _transform) in visuals.iter() {
            if let Some(model_id) = visual.model_resource_id() {
                model_ids.insert(*model_id);
            }
        }

        // json_mesh == model, json_primitive == mesh
        let model_ids = model_ids.into_iter().collect::<Vec<ResourceTypeAndId>>();
        for model_id in &model_ids {
            let model_meta_data = model_manager.get_model_meta_data(model_id).unwrap();

            let buffer_idx = self.buffers.len() as u32;

            let buffer_file_name = format!("buffer_{}.bin", model_id);
            let mut writer =
                std::fs::File::create(format!("{}/{}", self.output_dir, buffer_file_name))
                    .expect("I/O error");

            let mut byte_offset = 0;

            let mut json_primitives = Vec::new();
            for mesh in &model_meta_data.mesh_instances {
                let buffer_view_idx = self.buffer_views.len() as u32;
                let accessor_idx = self.accessors.len() as u32;

                let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
                let positions: Vec<Vec3> = mesh_meta_data
                    .source_data
                    .positions
                    .iter()
                    .map(|pos| Vec3::new(pos.x, pos.y, -pos.z))
                    .collect();
                let ptr = positions.as_ptr().cast::<u8>();
                let size_pos =
                    mesh_meta_data.source_data.positions.len() * std::mem::size_of::<Vec3>();
                writer
                    .write_all(unsafe { core::slice::from_raw_parts(ptr, size_pos) })
                    .unwrap();
                let buffer_view = gltf::json::buffer::View {
                    buffer: Index::new(buffer_idx),
                    byte_length: size_pos as u32,
                    byte_offset: Some(byte_offset),
                    byte_stride: None,
                    name: Some("Positions".into()),
                    target: None,
                    extensions: None,
                    extras: gltf::json::extras::Void::default(),
                };
                byte_offset += size_pos as u32;
                self.buffer_views.push(buffer_view);

                let accessor_pos = gltf::json::Accessor {
                    buffer_view: Some(Index::new(buffer_view_idx)),
                    byte_offset: 0,
                    count: mesh_meta_data.source_data.positions.len() as u32,
                    component_type: Checked::Valid(gltf::json::accessor::GenericComponentType(
                        gltf::json::accessor::ComponentType::F32,
                    )),
                    extensions: None,
                    extras: gltf::json::extras::Void::default(),
                    type_: Checked::Valid(gltf::json::accessor::Type::Vec3),
                    min: None,
                    max: None,
                    name: Some("Positions".into()),
                    normalized: false,
                    sparse: None,
                };
                self.accessors.push(accessor_pos);

                let mut attributes = HashMap::new();
                attributes.insert(
                    Checked::Valid(Semantic::Positions),
                    Index::new(accessor_idx),
                );

                let buffer_view_idx = self.buffer_views.len() as u32;
                let accessor_idx = self.accessors.len() as u32;

                let mut indices = None;

                if let Some(source_indices) = &mesh_meta_data.source_data.indices {
                    let mut source_indices = source_indices.clone();
                    for i in 0..source_indices.len() / 3 {
                        source_indices.swap(i * 3 + 1, i * 3 + 2);
                    }
                    let ptr = source_indices.as_ptr().cast::<u8>();
                    let size_indices = source_indices.len() * std::mem::size_of::<u16>();
                    writer
                        .write_all(unsafe { core::slice::from_raw_parts(ptr, size_indices) })
                        .unwrap();

                    let buffer_view = gltf::json::buffer::View {
                        buffer: Index::new(buffer_idx),
                        byte_length: size_indices as u32,
                        byte_offset: Some(byte_offset),
                        byte_stride: None,
                        name: Some("Indices".into()),
                        target: None,
                        extensions: None,
                        extras: gltf::json::extras::Void::default(),
                    };
                    byte_offset += size_indices as u32;
                    self.buffer_views.push(buffer_view);

                    let accessor_pos = gltf::json::Accessor {
                        buffer_view: Some(Index::new(buffer_view_idx)),
                        byte_offset: 0,
                        count: source_indices.len() as u32,
                        component_type: Checked::Valid(gltf::json::accessor::GenericComponentType(
                            gltf::json::accessor::ComponentType::U16,
                        )),
                        extensions: None,
                        extras: gltf::json::extras::Void::default(),
                        type_: Checked::Valid(gltf::json::accessor::Type::Scalar),
                        min: None,
                        max: None,
                        name: Some("Indices".into()),
                        normalized: false,
                        sparse: None,
                    };
                    self.accessors.push(accessor_pos);

                    indices = Some(Index::new(accessor_idx));
                }

                let primitive = gltf::json::mesh::Primitive {
                    attributes,
                    extensions: None,
                    extras: gltf::json::extras::Void::default(),
                    indices,
                    material: None,
                    mode: Checked::Valid(gltf::mesh::Mode::Triangles),
                    targets: None,
                };
                json_primitives.push(primitive);
            }

            // buffer file per model
            let buffer = gltf::json::Buffer {
                byte_length: byte_offset,
                name: None,
                uri: Some(buffer_file_name),
                extensions: None,
                extras: gltf::json::extras::Void::default(),
            };
            self.buffers.push(buffer);

            let json_mesh = gltf::json::Mesh {
                extensions: None,
                extras: gltf::json::extras::Void::default(),
                name: None,
                primitives: json_primitives,
                weights: None,
            };
            self.meshes.push(json_mesh);
        }
        model_ids
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn on_scene_export_requested(
    mut event_scene_export_requested: EventReader<'_, '_, SceneExportRequested>,
    visuals: Query<'_, '_, (&VisualComponent, &GlobalTransform), Without<ManipulatorComponent>>,
    lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    cameras: Query<'_, '_, &CameraComponent>,
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    render_surfaces: Query<'_, '_, &mut RenderSurface>,
) {
    if event_scene_export_requested.is_empty() {
        return;
    }
    let event = event_scene_export_requested.iter().next().unwrap();
    let dir = format!("{}/scene_export", event.export_path);
    if std::fs::create_dir_all(&dir).is_err() {
        error!("Couldn't create directory {dir}");
    }

    let mut ctx = SceneExportContext::new(&dir);

    let extensions = Some(ctx.export_extensions(&lights));
    let model_ids = ctx.export_models(&visuals, &model_manager, &mesh_manager);

    for (visual, transform) in visuals.iter() {
        if let Some(model_id) = visual.model_resource_id() {
            let node = gltf::json::Node {
                camera: None,
                children: None,
                extensions: None,
                extras: gltf::json::extras::Void::default(),
                matrix: None,
                mesh: Some(Index::new(
                    model_ids.iter().position(|v| *v == *model_id).unwrap() as u32,
                )),
                name: Some("mesh".into()),
                rotation: Some(gltf::json::scene::UnitQuaternion([
                    -transform.rotation.x,
                    -transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ])),
                scale: Some(transform.scale.into()),
                translation: Some([
                    transform.translation.x,
                    transform.translation.y,
                    -transform.translation.z,
                ]),
                skin: None,
                weights: None,
            };
            let node_idx = ctx.nodes.len() as u32;
            ctx.scene_nodes.push(Index::new(node_idx));
            ctx.nodes.push(node);
        }
    }

    let mut json_cameras = Vec::new();
    for camera in cameras.iter() {
        let camera_idx = json_cameras.len() as u32;

        let json_camera = gltf::json::Camera {
            name: None,
            orthographic: None,
            perspective: Some(gltf::json::camera::Perspective {
                aspect_ratio: {
                    //TODO: better way to grab aspect_ratio
                    let render_surface = render_surfaces.iter().next().unwrap();
                    Some(
                        render_surface.extents().width() as f32
                            / render_surface.extents().height() as f32,
                    )
                },
                yfov: camera.fov_y().radians(),
                zfar: Some(camera.z_far()),
                znear: camera.z_near(),
                extensions: None,
                extras: gltf::json::extras::Void::default(),
            }),
            type_: Checked::Valid(gltf::json::camera::Type::Perspective),
            extensions: None,
            extras: gltf::json::extras::Void::default(),
        };
        json_cameras.push(json_camera);

        let node = gltf::json::Node {
            camera: Some(Index::new(camera_idx)),
            children: None,
            extensions: None,
            extras: gltf::json::extras::Void::default(),
            matrix: None,
            mesh: None,
            name: Some(format!("Camera {}", camera_idx)),
            rotation: Some(gltf::json::scene::UnitQuaternion({
                let (yaw, pitch, _roll) = camera.rotation().to_euler(EulerRot::YXZ);
                lgn_math::Quat::from_euler(EulerRot::YXZ, -yaw + std::f32::consts::PI, pitch, 0.0)
                    .into()
            })),
            scale: None,
            translation: Some([
                camera.position().x,
                camera.position().y,
                -camera.position().z,
            ]),
            skin: None,
            weights: None,
        };
        let node_idx = ctx.nodes.len() as u32;
        ctx.scene_nodes.push(Index::new(node_idx));
        ctx.nodes.push(node);
    }

    let scene = gltf::json::Scene {
        extensions: None,
        extras: gltf::json::extras::Void::default(),
        name: Some("Legion scene".into()),
        nodes: ctx.scene_nodes,
    };

    let root = gltf::json::Root {
        extensions,
        accessors: ctx.accessors,
        //animations: todo!(),
        //asset: todo!(),
        buffers: ctx.buffers,
        buffer_views: ctx.buffer_views,
        //scene: todo!(),
        //extras: todo!(),
        extensions_used: vec!["KHR_lights_punctual".into()],
        extensions_required: vec!["KHR_lights_punctual".into()],
        cameras: json_cameras,
        //images: todo!(),
        //materials: todo!(),
        meshes: ctx.meshes,
        nodes: ctx.nodes,
        //samplers: todo!(),
        scenes: vec![scene],
        //skins: todo!(),
        //textures: todo!(),
        ..gltf::json::Root::default()
    };
    let writer = std::fs::File::create(format!("{}/scene.gltf", dir)).expect("I/O error");
    gltf::json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");
}

pub(crate) struct UiData {
    path: String,
    mul_x: f32,
    mul_y: f32,
    mul_z: f32,
    add_angle: f32,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            path: String::from("C:/scene_export"),
            mul_x: 1.0,
            mul_y: 1.0,
            mul_z: 1.0,
            add_angle: 0.0,
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_scene_exporter(
    egui: Res<'_, Egui>,
    mut event_scene_export_requested: ResMut<'_, Events<SceneExportRequested>>,
    mut ui_data: Local<'_, UiData>,
    cameras: Query<'_, '_, &CameraComponent>,
) {
    egui::Window::new("Scene export").show(&egui.ctx(), |ui| {
        ui.text_edit_singleline(&mut ui_data.path);
        ui.add(egui::Slider::new(&mut ui_data.mul_x, -1.0..=1.0).text("mul_x"));
        ui.add(egui::Slider::new(&mut ui_data.mul_y, -1.0..=1.0).text("mul_y"));
        ui.add(egui::Slider::new(&mut ui_data.mul_z, -1.0..=1.0).text("mul_z"));
        ui.add(
            egui::Slider::new(
                &mut ui_data.add_angle,
                -std::f32::consts::PI..=std::f32::consts::PI,
            )
            .text("add_angle"),
        );

        let (axis, angle) = cameras.iter().next().unwrap().rotation().to_axis_angle();
        ui.label(format!("Axis: {:?}, angle: {}", axis, angle));

        if ui.button("Export").clicked() {
            event_scene_export_requested.send(SceneExportRequested {
                export_path: ui_data.path.clone(),
                mul_x: ui_data.mul_x,
                mul_y: ui_data.mul_y,
                mul_z: ui_data.mul_z,
                add_angle: ui_data.add_angle,
            });
        }
    });
}

#[allow(dead_code)]
pub(crate) struct SceneExportRequested {
    export_path: String,
    mul_x: f32,
    mul_y: f32,
    mul_z: f32,
    add_angle: f32,
}
