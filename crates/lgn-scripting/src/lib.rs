//! Scripting library - currently using the MUN language

#[path = "../codegen/offline/mod.rs"]
#[cfg(feature = "offline")]
pub mod offline;

#[path = "../codegen/runtime/mod.rs"]
#[cfg(feature = "runtime")]
pub mod runtime;

pub mod components;

/*use lgn_ecs::prelude::*;

#[derive(Component)]
pub struct ScriptECSComponent {
    pub back_ref: ResourceId,
}*/

/*#[derive(Serialize, Deserialize)]
pub struct Script {

#[typetag::serde]
impl Component for Script {
    fn extract_build_deps(&self) -> Vec<ResourcePathId> {
        vec![]
    }
}*/

use std::{cell::RefCell, rc::Rc};

use components::ECSScriptComponent;
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use mun_runtime::{invoke_fn, Runtime, RuntimeBuilder};
use std::str::FromStr;

#[derive(Default)]
struct RuntimeScripts {
    pub runtimes: Vec<(ECSScriptComponent, Rc<RefCell<Runtime>>)>,
}

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<RuntimeScripts>()
            .add_system(Self::tick_scripts);
    }
}

impl ScriptingPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn tick_scripts(
        mut runtimes: NonSendMut<'_, RuntimeScripts>,
        scripts: Query<'_, '_, &mut ECSScriptComponent>,
    ) {
        if runtimes.runtimes.is_empty() {
            for script in scripts.iter() {
                let runtime = RuntimeBuilder::new(&script.lib_path)
                    .spawn()
                    .expect("Failed to spawn Runtime");
                runtimes.runtimes.push((script.clone(), runtime));
            }
        }
        for runtime in &runtimes.runtimes {
            {
                let runtime_ref = runtime.1.borrow();
                let result: i64 = invoke_fn!(
                    runtime_ref,
                    &runtime.0.entry_fn,
                    i64::from_str(&runtime.0.input_values[0]).unwrap()
                )
                .unwrap();
                println!("fibonacci({}) = {}", &runtime.0.input_values[0], result);
            }

            // reload the script of the path changed
            runtime.1.borrow_mut().update();
        }
    }
}
