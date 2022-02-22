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
    Context, ContextError, Diagnostics, Hash, Source, Sources, ToValue, Unit, Value, Vm,
};

mod modules;

use self::modules::{
    ecs::{make_ecs_module, Entity, EntityLookupByName},
    math::make_math_module,
    scripting::{make_scripting_module, Events},
    transform::make_transform_module,
};
use crate::{
    plugin::get_script, runtime::ScriptComponent, ScriptType, ScriptingEventCache, ScriptingStage,
};

pub(crate) fn build(app: &mut App) -> Result<(), ContextError> {
    let mut context = rune_modules::default_context()?;

    context.install(&make_scripting_module()?)?;
    context.install(&make_ecs_module()?)?;
    context.install(&make_transform_module()?)?;
    context.install(&make_math_module()?)?;

    app.init_non_send_resource::<VMCollection>()
        .insert_resource(context)
        .add_system_to_stage(ScriptingStage::Compile, compile)
        .add_system_to_stage(ScriptingStage::Execute, tick.exclusive_system());

    Ok(())
}

fn compile(
    scripts: Query<'_, '_, (lgn_ecs::prelude::Entity, &ScriptComponent)>,
    mut rune_vms: NonSendMut<'_, VMCollection>,
    rune_context: Res<'_, Context>,
    registry: Res<'_, Arc<AssetRegistry>>,
    mut commands: Commands<'_, '_>,
) {
    let rune_scripts = scripts
        .iter()
        .filter(|(_entity, s)| s.script_type == ScriptType::Rune);

    for (entity, script) in rune_scripts {
        let source_payload = &get_script(script, &registry).compiled_script;
        let source_payload = str::from_utf8(source_payload).unwrap();

        let mut sources = Sources::new();
        sources.insert(Source::new("entry", &source_payload));

        let mut diagnostics = Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&rune_context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            info!("script payload: {}", &source_payload);
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

    drop(scripts);
    drop(rune_context);
    drop(registry);
}

fn tick(world: &mut World) {
    // get all entities with a compiled Rune script
    let scripted_entities = world
        .query_filtered::<lgn_ecs::prelude::Entity, With<ScriptExecutionContext>>()
        .iter(world)
        .collect::<Vec<_>>();

    let world_ptr = world as *mut World;
    let event_cache =
        world.get_resource::<ScriptingEventCache>().unwrap() as *const ScriptingEventCache;

    for entity in scripted_entities {
        let mut args: Vec<Value> = Vec::new();

        let (script_vm_index, script_entry_fn) = {
            let script = world
                .entity(entity)
                .get::<ScriptExecutionContext>()
                .unwrap();

            for input in &script.input_args {
                if input == "{entity}" {
                    let entity = Entity::new(world_ptr, entity);
                    args.push(entity.to_value().unwrap());
                } else if input == "{events}" {
                    let events = Events::new(event_cache);
                    args.push(events.to_value().unwrap());
                } else if input == "{entities}" {
                    let entity_lookup = EntityLookupByName::new(world_ptr);
                    args.push(entity_lookup.to_value().unwrap());
                } else if input == "{result}" {
                    let rune_vms = world.get_non_send_resource::<VMCollection>().unwrap();
                    let vm_context = rune_vms.get(script.vm_index).as_ref().unwrap();
                    args.push(vm_context.last_result.clone());
                } else {
                    // default to 64-bit integer
                    let value = i64::from_str(input.as_str()).unwrap();
                    args.push(value.to_value().unwrap());
                }
            }

            (script.vm_index, script.entry_fn)
        };

        {
            let mut rune_vms = world.get_non_send_resource_mut::<VMCollection>().unwrap();
            let vm_context = rune_vms.get_mut(script_vm_index).as_mut().unwrap();
            let mut vm_exec = vm_context.vm.execute(script_entry_fn, args).unwrap();
            vm_context.last_result = vm_exec.complete().unwrap();
        }
    }
}

#[derive(Component)]
struct ScriptExecutionContext {
    vm_index: usize,
    entry_fn: Hash,
    input_args: Vec<String>,
}

#[derive(Default)]
struct VMCollection {
    vms: Vec<Option<VMContext>>,
}

struct VMContext {
    vm: Vm,
    last_result: Value,
}

impl VMCollection {
    fn append(&mut self, context: &Context, unit: Unit) -> usize {
        let vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
        self.vms.push(Some(VMContext {
            vm,
            last_result: ().to_value().unwrap(),
        }));
        self.vms.len() - 1
    }

    fn get(&self, index: usize) -> &Option<VMContext> {
        &self.vms[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Option<VMContext> {
        &mut self.vms[index]
    }
}
