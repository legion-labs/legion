//#[cfg(not(feature = "offline"))]
use std::sync::Arc;

use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
//#[cfg(not(feature = "offline"))]
use lgn_data_runtime::{AssetRegistry, ResourceTypeAndId};
use lgn_ecs::prelude::*;
//#[cfg(not(feature = "offline"))]
use lgn_input::mouse::MouseMotion;
//#[cfg(not(feature = "offline"))]
use lgn_math::prelude::*;

//#[cfg(not(feature = "offline"))]
use crate::{
    runtime::{Script, ScriptComponent},
    ScriptingStage,
};

//#[cfg(not(feature = "offline"))]
mod mun;
//#[cfg(not(feature = "offline"))]
mod rhai;
//#[cfg(not(feature = "offline"))]
mod rune;

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types.exclusive_system());
        app.add_startup_system(add_loaders);

        //#[cfg(not(feature = "offline"))]
        {
            app.add_stage(ScriptingStage::Compile, SystemStage::parallel());
            app.add_stage_after(
                ScriptingStage::Compile,
                ScriptingStage::Prepare,
                SystemStage::parallel(),
            );
            app.add_stage_after(
                ScriptingStage::Prepare,
                ScriptingStage::Execute,
                SystemStage::parallel(),
            );

            app.init_resource::<ScriptingEventCache>()
                .add_system_to_stage(ScriptingStage::Prepare, Self::update_events);

            mun::build(app);
            rune::build(app).expect("failed to setup Rune context");
            rhai::build(app);
        }
    }
}

impl ScriptingPlugin {
    //#[cfg(not(feature = "offline"))]
    pub(crate) fn update_events(
        mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
        mut cache: ResMut<'_, ScriptingEventCache>,
    ) {
        // aggregate mouse movement
        let mut delta = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }
        cache.mouse_motion.delta = delta;
    }
}

//#[cfg(not(feature = "offline"))]
#[derive(Clone)]
pub struct ScriptingEventCache {
    mouse_motion: MouseMotion,
}

//#[cfg(not(feature = "offline"))]
impl Default for ScriptingEventCache {
    fn default() -> Self {
        Self {
            mouse_motion: MouseMotion {
                delta: Vec2::default(),
            },
        }
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

//#[cfg(not(feature = "offline"))]
fn get_script_payload<'registry>(
    script: &ScriptComponent,
    registry: &'registry Res<'_, Arc<AssetRegistry>>,
) -> &'registry [u8] {
    let script_id = script.script_id.as_ref().unwrap().id();
    get_script_payload_by_id(script_id, registry)
}

//#[cfg(not(feature = "offline"))]
fn get_script_payload_by_id<'registry>(
    script_id: ResourceTypeAndId,
    registry: &'registry Res<'_, Arc<AssetRegistry>>,
) -> &'registry [u8] {
    let script_untyped = registry.get_untyped(script_id).unwrap();
    let script_typed = script_untyped.get::<Script>(&registry).unwrap();
    &script_typed.compiled_script
}
