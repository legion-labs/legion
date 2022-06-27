use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use crossbeam_channel::{Receiver, Sender};
use lgn_app::prelude::{App, CoreStage};
use lgn_data_runtime::prelude::*;
use lgn_ecs::{
    prelude::{IntoExclusiveSystem, NonSendMut, Res, With, World},
    system::EntityCommands,
};
use lgn_scripting_data::ScriptType;
use lgn_tracing::prelude::info;
use rune::{
    termcolor::{ColorChoice, StandardStream},
    Context, Diagnostics, Hash, Source, Sources, ToValue, Unit, Value, Vm,
};

mod modules;

use self::modules::{
    ecs::{make_ecs_module, Entity, EntityLookupByName},
    math::make_math_module,
    scripting::{make_scripting_module, Events},
    transform::make_transform_module,
};
use crate::{plugin::ScriptingEventCache, ScriptingStage};

pub(crate) struct ScriptComponentInstaller {
    sender: Sender<ScriptExecutionContextDestructionEvent>,
    rune_vms: Arc<RwLock<VMCollection>>,
    rune_context: Context,
}

impl ScriptComponentInstaller {
    fn new(
        sender: Sender<ScriptExecutionContextDestructionEvent>,
        rune_vms: Arc<RwLock<VMCollection>>,
    ) -> Self {
        let mut rune_context = rune_modules::default_context().unwrap();
        rune_context
            .install(&make_scripting_module().unwrap())
            .unwrap();
        rune_context.install(&make_ecs_module().unwrap()).unwrap();
        rune_context
            .install(&make_transform_module().unwrap())
            .unwrap();
        rune_context.install(&make_math_module().unwrap()).unwrap();
        Self {
            sender,
            rune_vms,
            rune_context,
        }
    }
}

#[async_trait::async_trait]
impl lgn_data_runtime::ComponentInstaller for ScriptComponentInstaller {
    fn install_component(
        &self,
        _asset_registry: &AssetRegistry,
        component: &dyn lgn_data_runtime::Component,
        commands: &mut EntityCommands<'_, '_, '_>,
    ) -> Result<(), lgn_data_runtime::AssetRegistryError> {
        if let Some(script_component) =
            component.downcast_ref::<lgn_scripting_data::runtime::ScriptComponent>()
        {
            if let Some(handle) = script_component.script_id.as_ref().and_then(|reference| {
                reference.get_active_handle::<lgn_scripting_data::runtime::Script>()
            }) {
                if let Some(script_resource) = handle.get() {
                    if script_resource.script_type == ScriptType::Rune {
                        let source_payload = &script_resource.script;

                        let mut sources = Sources::new();
                        sources.insert(Source::new("entry", &source_payload));

                        let mut diagnostics = Diagnostics::new();

                        let result = rune::prepare(&mut sources)
                            .with_context(&self.rune_context)
                            .with_diagnostics(&mut diagnostics)
                            .build();

                        if !diagnostics.is_empty() {
                            info!("script payload: {}", &source_payload);
                            let mut writer = StandardStream::stderr(ColorChoice::Always);
                            diagnostics.emit(&mut writer, &sources).unwrap();
                        }

                        let unit = result.unwrap();

                        let vm_index = self
                            .rune_vms
                            .write()
                            .unwrap()
                            .append(&self.rune_context, unit);

                        let fn_name = &[script_component.entry_fn.as_str()];
                        let script_exec = ScriptExecutionContext {
                            vm_index,
                            entry_fn: Hash::type_hash(fn_name),
                            input_args: script_component.input_values.clone(),
                            event_sender: self.sender.clone(),
                        };
                        commands.insert(script_exec);
                    } else {
                        lgn_tracing::warn!("Unsupported script type {:?}", handle.id());
                    }
                }
            } else if let Some(script_id) = &script_component.script_id {
                lgn_tracing::warn!("Cannot find Script: {:?}", script_id.id());
            }
        }
        Ok(())
    }
}

pub(crate) fn build(app: &mut App) {
    let (sender, receiver) =
        crossbeam_channel::unbounded::<ScriptExecutionContextDestructionEvent>();

    let vm_collection = Arc::new(RwLock::new(VMCollection::default()));
    app.add_startup_system(setup);
    app.insert_resource(Arc::new(ScriptComponentInstaller::new(
        sender,
        vm_collection.clone(),
    )));

    app.insert_resource(vm_collection)
        .insert_resource(receiver)
        .add_system_to_stage(ScriptingStage::Execute, tick.exclusive_system())
        .add_system_to_stage(CoreStage::Last, cleanup_execution_contexts);
}

fn setup(
    asset_registry_options: NonSendMut<'_, AssetRegistryOptions>,
    script_component_installer: Res<'_, Arc<ScriptComponentInstaller>>,
) {
    let asset_registry_options = asset_registry_options.into_inner();
    asset_registry_options.add_component_installer(
        &[std::any::TypeId::of::<
            lgn_scripting_data::runtime::ScriptComponent,
        >()],
        script_component_installer.clone(),
    );
    drop(script_component_installer);
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
                    let rune_vms = world.get_resource::<Arc<RwLock<VMCollection>>>().unwrap();
                    if let Ok(readlock) = rune_vms.read() {
                        let vm_context = readlock.get(script.vm_index).as_ref().unwrap();
                        args.push(vm_context.last_result.clone());
                    }
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
            let rune_vms = world.get_resource::<Arc<RwLock<VMCollection>>>().unwrap();
            if let Ok(mut lock) = rune_vms.write() {
                let vm_context = lock.get_mut(script_vm_index).as_mut().unwrap();
                let mut vm_exec = vm_context.vm.execute(script_entry_fn, args).unwrap();
                vm_context.last_result = vm_exec.complete().unwrap();
            }
        }
    }
}

#[derive(lgn_ecs::prelude::Component)]
struct ScriptExecutionContext {
    vm_index: usize,
    entry_fn: Hash,
    input_args: Vec<String>,
    event_sender: Sender<ScriptExecutionContextDestructionEvent>,
}

impl Drop for ScriptExecutionContext {
    fn drop(&mut self) {
        let _result = self.event_sender.send((&*self).into());
    }
}

fn cleanup_execution_contexts(
    receiver: Res<'_, Receiver<ScriptExecutionContextDestructionEvent>>,
    rune_vms: Res<'_, Arc<RwLock<VMCollection>>>,
) {
    for event in receiver.try_iter() {
        rune_vms.write().unwrap().remove(event.vm_index);
    }

    drop(receiver);
    drop(rune_vms);
}

struct ScriptExecutionContextDestructionEvent {
    vm_index: usize,
}

impl From<&ScriptExecutionContext> for ScriptExecutionContextDestructionEvent {
    fn from(script_exec: &ScriptExecutionContext) -> Self {
        Self {
            vm_index: script_exec.vm_index,
        }
    }
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

// TODO: HACK Find a better way of fixing this
#[allow(unsafe_code)]
unsafe impl Send for VMContext {}
#[allow(unsafe_code)]
unsafe impl Sync for VMContext {}

struct VMContext {
    vm: Vm,
    last_result: Value,
}
