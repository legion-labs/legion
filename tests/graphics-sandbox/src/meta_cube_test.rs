use std::{fs::File, io, path::Path, sync::Arc};

use lgn_app::{App, CoreStage, Plugin, StartupStage};
use lgn_data_runtime::{AssetRegistry, AssetRegistryScheduling};
use lgn_ecs::{
    prelude::{Commands, Entity, Query, Res},
    schedule::ParallelSystemDescriptorCoercion,
};
use lgn_graphics_renderer::{
    components::VisualComponent,
    resources::{RenderModel, CUBE_MODEL_RESOURCE_ID},
};
use lgn_math::{Quat, Vec3};
use lgn_tracing::span_fn;
use lgn_transform::prelude::{GlobalTransform, Transform, TransformBundle};
use png::OutputInfo;

const META_CUBE_SIZE: usize = 10;

#[derive(Default)]
pub struct MetaCubePlugin {
    meta_cube_size: usize,
}

impl MetaCubePlugin {
    pub fn new() -> Self {
        Self {
            meta_cube_size: META_CUBE_SIZE,
        }
    }
}

impl Plugin for MetaCubePlugin {
    fn build(&self, app: &mut App) {
        if self.meta_cube_size != 0 {
            app.insert_resource(MetaCubeResource::new(self.meta_cube_size));

            app.add_system_to_stage(CoreStage::PostUpdate, modify_transform_data);

            app.add_startup_system_to_stage(
                StartupStage::PostStartup,
                init_stress_test.after(AssetRegistryScheduling::AssetRegistryCreated),
            );
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_stress_test(
    commands: Commands<'_, '_>,
    meta_cube: Res<'_, MetaCubeResource>,
    asset_registry: Res<'_, Arc<AssetRegistry>>,
) {
    meta_cube.initialize(commands, asset_registry.as_ref());
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn modify_transform_data(
    mut query: Query<'_, '_, (Entity, &mut GlobalTransform, &VisualComponent)>,
) {
    for (_, mut transform, _) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(Vec3::Y, 0.2));
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ColorData {
    pub(crate) data: Vec<u8>,
    pub(crate) info: OutputInfo,
}

/// Load the image using `png`
pub(crate) fn load_image(path: &Path) -> io::Result<ColorData> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut img_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut img_data)?;

    Ok(ColorData {
        data: img_data,
        info,
    })
}

struct MetaCubeResource {
    meta_cube_size: usize,
}

impl MetaCubeResource {
    pub fn new(meta_cube_size: usize) -> Self {
        Self { meta_cube_size }
    }

    #[allow(clippy::cast_precision_loss)]
    fn initialize(&self, mut commands: Commands<'_, '_>, asset_registry: &AssetRegistry) {
        let ref_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("refs")
            .join("stress-test")
            .join("random_color")
            .with_extension("png");

        let random_color = load_image(&ref_path).unwrap();

        let render_model_handle = asset_registry
            .lookup::<RenderModel>(&CUBE_MODEL_RESOURCE_ID)
            .expect("Must be loaded");

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
                        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
                            x as f32 * 2.0,
                            y as f32 * 2.0,
                            z as f32 * 2.0,
                        )))
                        .insert(VisualComponent::new(
                            &render_model_handle,
                            (r, g, b).into(),
                            1.0,
                        ));
                }
            }
        }
    }
}
