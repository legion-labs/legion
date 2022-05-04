use std::{str::FromStr, sync::Arc};

use lgn_app::prelude::{App, CoreStage};
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::prelude::{
    Commands, Component, IntoExclusiveSystem, NonSendMut, Query, Res, With, Without, World,
};
use lgn_scripting_data::{runtime::ScriptComponent, ScriptType};
use lgn_tracing::prelude::{error, info};
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
    plugin::{get_script, ScriptingEventCache},
    ScriptingStage,
};

pub(crate) fn build(app: &mut App) -> Result<(), ContextError> {
    let mut context = rune_modules::default_context()?;

    context.install(&make_scripting_module()?)?;
    context.install(&make_ecs_module()?)?;
    context.install(&make_transform_module()?)?;
    context.install(&make_math_module()?)?;

    let (sender, receiver) =
        crossbeam_channel::unbounded::<ScriptExecutionContextDestructionEvent>();

    app.init_non_send_resource::<VMCollection>()
        .insert_resource(context)
        .insert_resource(sender)
        .insert_resource(receiver)
        .add_system_to_stage(ScriptingStage::Compile, compile)
        .add_system_to_stage(ScriptingStage::Execute, tick.exclusive_system())
        .add_system_to_stage(CoreStage::Last, cleanup);

    Ok(())
}

fn compile(
    scripts: Query<
        '_,
        '_,
        (lgn_ecs::prelude::Entity, &ScriptComponent),
        Without<ScriptExecutionContext>,
    >,
    mut rune_vms: NonSendMut<'_, VMCollection>,
    rune_context: Res<'_, Context>,
    sender: Res<'_, crossbeam_channel::Sender<ScriptExecutionContextDestructionEvent>>,
    registry: Res<'_, Arc<AssetRegistry>>,
    mut commands: Commands<'_, '_>,
) {
    for (entity, script) in scripts.iter() {
        let script_resource = get_script(script, &registry);
        if script_resource.script_type != ScriptType::Rune {
            continue;
        }

        let source_payload = &script_resource.script;

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
            event_writer: sender.clone(),
        };

        commands.entity(entity).insert(script_exec);
    }

    drop(scripts);
    drop(rune_context);
    drop(registry);
    drop(sender);
}

fn tick(world: &mut World) {
    // get all entities with a compiled Rune script
    let scripted_entities = world
        .query_filtered::<lgn_ecs::prelude::Entity, With<ScriptExecutionContext>>()
        .iter(world)
        .collect::<Vec<_>>();

    let world_ptr = world as *mut World;
    let event_cache = world.resource::<ScriptingEventCache>() as *const ScriptingEventCache;

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
                    let rune_vms = world.non_send_resource::<VMCollection>();
                    let vm_context = rune_vms.get(script.vm_index).as_ref().unwrap();
                    args.push(vm_context.last_result.clone());
                } else {
                    // try primitive types
                    let input = input.as_str();
                    if let Ok(value) = i64::from_str(input) {
                        args.push(value.to_value().unwrap());
                    } else if let Ok(value) = f64::from_str(input) {
                        args.push(value.to_value().unwrap());
                    } else {
                        args.push(input.to_value().unwrap());
                    }
                }
            }

            (script.vm_index, script.entry_fn)
        };

        {
            let mut rune_vms = world.non_send_resource_mut::<VMCollection>();
            let vm_context = rune_vms.get_mut(script_vm_index).as_mut().unwrap();
            let mut vm_exec = vm_context.vm.execute(script_entry_fn, args).unwrap();
            vm_context.last_result = vm_exec.complete().unwrap();
        }
    }
}

fn cleanup(
    receiver: Res<'_, crossbeam_channel::Receiver<ScriptExecutionContextDestructionEvent>>,
    mut rune_vms: NonSendMut<'_, VMCollection>,
) {
    while let Ok(event) = receiver.try_recv() {
        rune_vms.remove(event.vm_index);
    }

    drop(receiver);
}

#[derive(Component)]
struct ScriptExecutionContext {
    vm_index: usize,
    entry_fn: Hash,
    input_args: Vec<String>,
    event_writer: crossbeam_channel::Sender<ScriptExecutionContextDestructionEvent>,
}

impl Drop for ScriptExecutionContext {
    fn drop(&mut self) {
        if let Err(e) = self
            .event_writer
            .send(ScriptExecutionContextDestructionEvent {
                vm_index: self.vm_index,
            })
        {
            error!(
                "failed to send ScriptExecutionContextDestructionEvent: {}",
                e
            );
        }
    }
}

struct ScriptExecutionContextDestructionEvent {
    vm_index: usize,
}

#[derive(Default)]
struct VMCollection {
    vms: Vec<Option<VMContext>>,
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

    fn remove(&mut self, index: usize) {
        self.vms[index].take();
    }

    fn get(&self, index: usize) -> &Option<VMContext> {
        &self.vms[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Option<VMContext> {
        &mut self.vms[index]
    }
}

struct VMContext {
    vm: Vm,
    last_result: Value,
}

impl Drop for VMContext {
    fn drop(&mut self) {
        info!("VMContext dropped");
    }
}
