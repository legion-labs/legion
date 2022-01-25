use std::{fs::File, io, path::Path};

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Commands, Query, Res};
use lgn_math::{EulerRot, Quat};
use lgn_tracing::span_fn;
use lgn_transform::components::Transform;

use crate::components::{RotationComponent, StaticMesh};

use super::{DefaultMaterialType, DefaultMeshType, DefaultMeshes};

#[derive(Default)]
pub struct MetaCubePlugin {
    meta_cube_size: usize,
}

impl MetaCubePlugin {
    pub fn new(meta_cube_size: usize) -> Self {
        Self { meta_cube_size }
    }
}

impl Plugin for MetaCubePlugin {
    fn build(&self, app: &mut App) {
        if self.meta_cube_size != 0 {
            app.insert_resource(MetaCubeResource::new(self.meta_cube_size));

            app.add_startup_system(init_stress_test);

            app.add_system(update_rotation);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_stress_test(
    commands: Commands<'_, '_>,
    default_meshes: Res<'_, DefaultMeshes>,
    meta_cube: Res<'_, MetaCubeResource>,
) {
    meta_cube.initialize(commands, &default_meshes);
}

#[span_fn]
fn update_rotation(mut query: Query<'_, '_, (&mut Transform, &RotationComponent)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.rotation_speed.0 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.1 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.2 / 60.0 * std::f32::consts::PI,
        ));
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ColorData {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

/// Load the image using `png`
pub(crate) fn load_image(path: &Path) -> io::Result<ColorData> {
    use png::ColorType::Rgb;
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut img_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut img_data)?;

    match info.color_type {
        Rgb => Ok(ColorData {
            data: img_data,
            width: info.width,
            height: info.height,
        }),
        _ => unreachable!("uncovered color type"),
    }
}

struct MetaCubeResource {
    meta_cube_size: usize,
}

impl MetaCubeResource {
    pub fn new(meta_cube_size: usize) -> Self {
        Self { meta_cube_size }
    }

    fn initialize(&self, mut commands: Commands<'_, '_>, default_meshes: &DefaultMeshes) {
        let ref_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("random_color")
            .with_extension("png");

        let random_color = load_image(&ref_path).unwrap();

        for x in 0..self.meta_cube_size {
            for y in 0..self.meta_cube_size {
                for z in 0..self.meta_cube_size {
                    let flattened_index = (x * self.meta_cube_size * self.meta_cube_size)
                        + y * self.meta_cube_size
                        + z;

                    let r = random_color.data[flattened_index * 3];
                    let g = random_color.data[flattened_index * 3 + 1];
                    let b = random_color.data[flattened_index * 3 + 2];

                    commands
                        .spawn()
                        .insert(Transform::from_xyz(
                            x as f32 * 2.0,
                            y as f32 * 2.0,
                            z as f32 * 2.0,
                        ))
                        .insert(StaticMesh::from_default_meshes(
                            default_meshes,
                            DefaultMeshType::Cube as usize,
                            (r, g, b).into(),
                            DefaultMaterialType::Default,
                        ))
                        .insert(RotationComponent {
                            rotation_speed: (0.0, 0.1 * ((flattened_index % 10) + 1) as f32, 0.0),
                        });
                }
            }
        }
    }
}
