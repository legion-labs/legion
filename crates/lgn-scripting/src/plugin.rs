use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::AssetRegistryOptions;
use lgn_ecs::prelude::*;

use std::{cell::RefCell, rc::Rc};

use crate::runtime::ScriptComponent;
use mun_runtime::{invoke_fn, Runtime};
use std::str::FromStr;

#[derive(Default)]
struct RuntimeScripts {
    pub runtimes: Vec<(ScriptComponent, Rc<RefCell<Runtime>>)>,
}

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types.exclusive_system());

        app.add_startup_system(add_loaders);

        app.init_non_send_resource::<RuntimeScripts>()
            .add_system(Self::tick_scripts);
    }
}

impl ScriptingPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn tick_scripts(
        runtimes: NonSendMut<'_, RuntimeScripts>,
        scripts: Query<'_, '_, &mut ScriptComponent>,
    ) {
        if runtimes.runtimes.is_empty() {
            for _script in scripts.iter() {
                /*TODO: Fix using compiled_script
                let runtime = RuntimeBuilder::new(&script.lib_path)
                    .spawn()
                    .expect("Failed to spawn Runtime");
                runtimes.runtimes.push((script.clone(), runtime));*/
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
