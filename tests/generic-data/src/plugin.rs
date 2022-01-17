use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;
#[cfg(not(feature = "offline"))]
use lgn_math::{prelude::*, EulerRot};

#[derive(Default)]
pub struct GenericDataPlugin;

impl Plugin for GenericDataPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types.exclusive_system());

        app.add_startup_system(add_loaders);

        #[cfg(not(feature = "offline"))]
        app.add_system_to_stage(CoreStage::PreUpdate, update_rotation);
    }
}

#[cfg(feature = "offline")]
fn register_resource_types(world: &mut World) {
    if let Some(resource_registry) = world.get_non_send_resource_mut::<ResourceRegistryOptions>() {
        crate::offline::register_resource_types(resource_registry.into_inner());
    }
}

#[allow(unused_variables)]
fn add_loaders(asset_registry: NonSendMut<'_, AssetRegistryOptions>) {
    let asset_registry = asset_registry.into_inner();
    #[cfg(feature = "offline")]
    {
        crate::offline::add_loaders(asset_registry);
    }

    #[cfg(feature = "runtime")]
    {
        crate::runtime::add_loaders(asset_registry);
    }
}

#[cfg(not(feature = "offline"))]
fn update_rotation(
    mut query: Query<
        '_,
        '_,
        (
            &mut lgn_transform::prelude::Transform,
            &lgn_renderer::components::RotationComponent,
        ),
    >,
) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.rotation_speed.0 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.1 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.2 / 60.0 * std::f32::consts::PI,
        ));
    }
}
