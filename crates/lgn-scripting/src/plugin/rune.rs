use std::{
    str::{self, FromStr},
    sync::Arc,
};

use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::prelude::*;
use lgn_tracing::prelude::*;
use rune::{
    termcolor::{ColorChoice, StandardStream},
    Context, Diagnostics, Hash, Source, Sources, ToValue, Unit, Value, Vm,
};

use crate::{
    runtime::{Script, ScriptComponent},
    ScriptType, ScriptingEventCache,
};

pub(crate) fn build(app: &mut App) {
    let context = rune_modules::default_context().unwrap();

    app.init_non_send_resource::<VMCollection>()
        .insert_resource(context)
        .add_system(compile)
        .add_system(tick);
}

fn compile(
    scripts: Query<'_, '_, (Entity, &mut ScriptComponent)>,
    mut rune_vms: NonSendMut<'_, VMCollection>,
    rune_context: Res<'_, Context>,
    registry: Res<'_, Arc<AssetRegistry>>,
    mut commands: Commands<'_, '_>,
) {
    let rune_scripts = scripts
        .iter()
        .filter(|(_entity, s)| s.script_type == ScriptType::Rune);

    for (entity, script) in rune_scripts {
        let script_untyped = registry.get_untyped(script.script_id.as_ref().unwrap().id());
        let script_typed = script_untyped.unwrap().get::<Script>(&registry).unwrap();
        let source_payload = str::from_utf8(&script_typed.compiled_script).unwrap();
        info!("script payload: {}", &source_payload);

        let mut sources = Sources::new();
        sources.insert(Source::new("entry", &source_payload));

        let mut diagnostics = Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&rune_context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }

        let unit = result.unwrap();

        let vm_index = rune_vms.append(&rune_context, unit);

        let fn_name = &[script.entry_fn.as_str()];
        let script_exec = ScriptExecutionContext {
            vm_index,
            entry_fn: Hash::type_hash(fn_name),
            input_args: script.input_values.clone(),
        };

        commands
            .entity(entity)
            .insert(script_exec)
            .remove::<ScriptComponent>();
    }
}

fn tick(
    mut rune_vms: NonSendMut<'_, VMCollection>,
    query: Query<'_, '_, (Entity, &mut ScriptExecutionContext)>,
    event_cache: Res<'_, ScriptingEventCache>,
) {
    for (_entity, script) in query.iter() {
        if let Some(vm) = rune_vms.get_mut(script.vm_index) {
            let mut args: Vec<Value> = Vec::new();
            for input in &script.input_args {
                if input == "mouse_delta_x" {
                    args.push(event_cache.mouse_motion.delta.x.to_value().unwrap());
                } else if input == "mouse_delta_y" {
                    args.push(event_cache.mouse_motion.delta.y.to_value().unwrap());
                } else {
                    let value = i64::from_str(input.as_str()).unwrap();
                    args.push(value.to_value().unwrap());
                }
            }

            let _result = vm
                .execute(script.entry_fn, args)
                .unwrap()
                .complete()
                .unwrap();
        }
    }

    drop(query);
    drop(event_cache);
}

#[derive(Component)]
struct ScriptExecutionContext {
    vm_index: usize,
    entry_fn: Hash,
    input_args: Vec<String>,
}

#[derive(Default)]
struct VMCollection {
    vms: Vec<Option<Vm>>,
}

impl VMCollection {
    fn append(&mut self, context: &Context, unit: Unit) -> usize {
        let vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
        self.vms.push(Some(vm));
        self.vms.len() - 1
    }

    fn get_mut(&mut self, index: usize) -> &mut Option<Vm> {
        &mut self.vms[index]
    }
}
