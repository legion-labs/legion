use std::{fs::File, io, path::Path};

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Commands, Res};
use lgn_renderer::components::StaticMesh;
use lgn_transform::components::{GlobalTransform, Transform};

use super::{DefaultMaterialType, DefaultMeshType, MeshManager};

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
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_stress_test(
    commands: Commands<'_, '_>,
    mesh_manager: Res<'_, MeshManager>,
    meta_cube: Res<'_, MetaCubeResource>,
) {
    meta_cube.initialize(commands, &mesh_manager);
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

    #[allow(clippy::cast_precision_loss)]
    fn initialize(&self, mut commands: Commands<'_, '_>, mesh_manager: &MeshManager) {
        let ref_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("refs")
            .join("stress-test")
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
                        .insert(GlobalTransform::identity())
                        .insert(StaticMesh::from_default_meshes(
                            mesh_manager,
                            DefaultMeshType::Cube as usize,
                            (r, g, b).into(),
                            DefaultMaterialType::Default,
                        ));
                }
            }
        }
    }
}
