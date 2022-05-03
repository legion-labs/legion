use std::sync::Arc;

use lgn_app::prelude::{App, Plugin};
use lgn_data_runtime::{AssetRegistry, AssetRegistryGuard};
use lgn_ecs::prelude::{EventReader, Res, ResMut, SystemStage};
use lgn_input::mouse::MouseMotion;
use lgn_math::prelude::Vec2;
use lgn_scripting_data::runtime::{Script, ScriptComponent};

use crate::ScriptingStage;

mod rhai;

mod rune;

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
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

        rune::build(app).expect("failed to setup Rune context");
        rhai::build(app);
    }
}

impl ScriptingPlugin {
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

#[derive(Clone)]
pub struct ScriptingEventCache {
    mouse_motion: MouseMotion,
}

impl Default for ScriptingEventCache {
    fn default() -> Self {
        Self {
            mouse_motion: MouseMotion { delta: Vec2::ZERO },
        }
    }
}

fn get_script<'registry>(
    script: &ScriptComponent,
    registry: &'registry Res<'_, Arc<AssetRegistry>>,
) -> AssetRegistryGuard<'registry, Script> {
    let script_id = script.script_id.as_ref().unwrap().id();
    let script_untyped = registry.get_untyped(script_id).unwrap();
    script_untyped.get::<Script>(registry).unwrap()
}
