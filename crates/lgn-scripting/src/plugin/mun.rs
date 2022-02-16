use std::{cell::RefCell, fs, rc::Rc, str::FromStr, sync::Arc};

use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::prelude::*;

use crate::{
    plugin::get_source_payload_by_id, runtime::ScriptComponent, ScriptType, ScriptingStage,
};

pub(crate) fn build(app: &mut App) {
    app.init_non_send_resource::<RuntimeCollection>()
        .add_system_to_stage(ScriptingStage::Compile, compile)
        .add_system_to_stage(ScriptingStage::Execute, tick);
}

fn compile(
    scripts: Query<'_, '_, (Entity, &ScriptComponent)>,
    mut runtime_collection: NonSendMut<'_, RuntimeCollection>,
    registry: Res<'_, Arc<AssetRegistry>>,
    mut commands: Commands<'_, '_>,
) {
    let mun_scripts = scripts
        .iter()
        .filter(|(_entity, s)| s.script_type == ScriptType::Mun);

    for (entity, script) in mun_scripts {
        let script_id = script.script_id.as_ref().unwrap().id();
        let source_payload = get_source_payload_by_id(script_id, &registry);

        let lib_path = {
            let mut temp_crate = std::env::temp_dir();
            temp_crate.push(script_id.id.to_string());
            fs::remove_dir_all(&temp_crate).unwrap_or_default();
            fs::create_dir_all(&temp_crate).unwrap();
            temp_crate.push("mod.munlib");
            fs::write(&temp_crate, source_payload).unwrap();
            temp_crate
        };
        println!("{:?}", &lib_path);

        let runtime = mun_runtime::RuntimeBuilder::new(&lib_path)
            .spawn()
            .expect("Failed to spawn Runtime");

        let script_exec = ScriptExecutionContext {
            runtime_index: runtime_collection.append(runtime),
            entry_fn: script.entry_fn.clone(),
            input_values: script.input_values.clone(),
        };

        commands
            .entity(entity)
            .insert(script_exec)
            .remove::<ScriptComponent>();
    }

    drop(scripts);
    drop(registry);
}

fn tick(
    query: Query<'_, '_, &ScriptExecutionContext>,
    runtime_collection: NonSend<'_, RuntimeCollection>,
) {
    for script in query.iter() {
        {
            let runtime_ref = runtime_collection.get(script.runtime_index).borrow();
            let arg = i64::from_str(&script.input_values[0]).unwrap();
            let result: i64 = mun_runtime::invoke_fn!(runtime_ref, &script.entry_fn, arg).unwrap();
            println!("Mun: fibonacci({}) = {}", &arg, result);
        }

        // reload the script of the path changed
        runtime_collection
            .get(script.runtime_index)
            .borrow_mut()
            .update();
    }

    drop(query);
    drop(runtime_collection);
}

#[derive(Component)]
struct ScriptExecutionContext {
    runtime_index: usize,
    entry_fn: String,
    input_values: Vec<String>,
}

#[derive(Default)]
struct RuntimeCollection {
    runtimes: Vec<Rc<RefCell<mun_runtime::Runtime>>>,
}

impl RuntimeCollection {
    fn append(&mut self, runtime: Rc<RefCell<mun_runtime::Runtime>>) -> usize {
        self.runtimes.push(runtime);
        self.runtimes.len() - 1
    }

    fn get(&self, index: usize) -> &Rc<RefCell<mun_runtime::Runtime>> {
        &self.runtimes[index]
    }
}
