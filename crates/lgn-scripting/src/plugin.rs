use std::{cell::RefCell, fs, rc::Rc, str::FromStr, sync::Arc};

use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};
use lgn_ecs::prelude::*;
use lgn_input::mouse::MouseMotion;
use lgn_math::prelude::*;
use lgn_tracing::prelude::*;
use rhai::Scope;
use rune::{
    termcolor::{ColorChoice, StandardStream},
    ToValue,
};

use crate::runtime::{Script, ScriptComponent};

struct RuntimeScripts {
    mun_runtimes: Vec<(ScriptComponent, Rc<RefCell<mun_runtime::Runtime>>)>,
    rune_context: rune::Context,
    rune_vms: RuneVMCollection,
    rhai_eng: Option<rhai::Engine>,
    rhai_asts: Vec<(ScriptComponent, Rc<RefCell<rhai::AST>>)>,
}

impl Default for RuntimeScripts {
    fn default() -> Self {
        Self {
            mun_runtimes: Vec::new(),
            rune_context: rune_modules::default_context().unwrap(),
            rune_vms: RuneVMCollection::default(),
            rhai_eng: None,
            rhai_asts: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "offline")]
        app.add_startup_system(register_resource_types.exclusive_system());

        app.init_non_send_resource::<RuntimeScripts>()
            .init_resource::<ScriptingEventCache>()
            .add_startup_system(add_loaders)
            .add_system(Self::update_events)
            .add_system(Self::tick_scripts)
            .add_system(Self::tick_rune);
    }
}

impl ScriptingPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn tick_scripts(
        runtimes: NonSendMut<'_, RuntimeScripts>,
        scripts: Query<'_, '_, (Entity, &mut ScriptComponent)>,
        registry: Res<'_, Arc<AssetRegistry>>,
        mut commands: Commands<'_, '_>,
    ) {
        let mun_components = scripts
            .iter()
            .filter(|(_entity, s)| s.script_type == 1 /*ScriptType::Mun*/);
        let rune_components = scripts
            .iter()
            .filter(|(_entity, s)| s.script_type == 2 /*ScriptType::Rune*/);
        let rhai_components = scripts
            .iter()
            .filter(|(_entity, s)| s.script_type == 3 /*ScriptType::Rhai*/);

        let r = runtimes.into_inner();
        Self::tick_mun(mun_components, r, &registry);
        Self::compile_rune(rune_components, r, &registry, &mut commands);
        Self::tick_rhai(rhai_components, r, &registry);
    }

    fn tick_mun<'a>(
        mun_components: impl Iterator<Item = (Entity, &'a ScriptComponent)>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
    ) {
        if runtimes.mun_runtimes.is_empty() {
            for (_entity, script) in mun_components {
                let script_id = script.script_id.as_ref().unwrap().id();
                let script_untyped = registry.get_untyped(script_id);
                let script_typed = script_untyped.unwrap().get::<Script>(registry).unwrap();

                let lib_path = {
                    let mut temp_crate = std::env::temp_dir();
                    temp_crate.push(script_id.id.to_string());
                    fs::remove_dir_all(&temp_crate).unwrap_or_default();
                    fs::create_dir_all(&temp_crate).unwrap();
                    temp_crate.push("mod.munlib");
                    fs::write(&temp_crate, &script_typed.compiled_script).unwrap();
                    temp_crate
                };
                println!("{:?}", &lib_path);

                let runtime = mun_runtime::RuntimeBuilder::new(&lib_path)
                    .spawn()
                    .expect("Failed to spawn Runtime");
                runtimes.mun_runtimes.push((script.clone(), runtime));
            }
        }
        for runtime in &runtimes.mun_runtimes {
            {
                let runtime_ref = runtime.1.borrow();
                let arg = i64::from_str(&runtime.0.input_values[0]).unwrap();
                let result: i64 =
                    mun_runtime::invoke_fn!(runtime_ref, &runtime.0.entry_fn, arg).unwrap();
                println!("Mun: fibonacci({}) = {}", &arg, result);
            }

            // reload the script of the path changed
            runtime.1.borrow_mut().update();
        }
    }

    fn compile_rune<'a>(
        rune_components: impl Iterator<Item = (Entity, &'a ScriptComponent)>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
        commands: &mut Commands<'_, '_>,
    ) {
        for (entity, script) in rune_components {
            let script_untyped = registry.get_untyped(script.script_id.as_ref().unwrap().id());
            let script_typed = script_untyped.unwrap().get::<Script>(registry).unwrap();
            let source_payload = std::str::from_utf8(&script_typed.compiled_script).unwrap();
            info!("script payload: {}", &source_payload);

            let mut sources = rune::Sources::new();
            sources.insert(rune::Source::new("entry", &source_payload));

            let mut diagnostics = rune::Diagnostics::new();

            let result = rune::prepare(&mut sources)
                .with_context(&runtimes.rune_context)
                .with_diagnostics(&mut diagnostics)
                .build();

            if !diagnostics.is_empty() {
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.emit(&mut writer, &sources).unwrap();
            }

            let unit = result.unwrap();

            let vm_index = runtimes
                .rune_vms
                .append_new_vm(&runtimes.rune_context, unit);

            let fn_name = &[script.entry_fn.as_str()];
            let script_exec = RuneScriptExecutionComponent {
                vm_index,
                entry_fn: rune::Hash::type_hash(fn_name),
                input_args: script.input_values.clone(),
            };

            commands
                .entity(entity)
                .insert(script_exec)
                .remove::<ScriptComponent>();
        }
    }

    fn tick_rune(
        mut runtimes: NonSendMut<'_, RuntimeScripts>,
        query: Query<'_, '_, (Entity, &mut RuneScriptExecutionComponent)>,
        event_cache: Res<'_, ScriptingEventCache>,
    ) {
        for (_entity, script) in query.iter() {
            if let Some(vm) = runtimes.rune_vms.get_mut(script.vm_index) {
                let mut args: Vec<rune::Value> = Vec::new();
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

    fn tick_rhai<'a>(
        rhai_components: impl Iterator<Item = (Entity, &'a ScriptComponent)>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
    ) {
        if runtimes.rhai_eng.is_none() {
            for (_entity, script) in rhai_components {
                if runtimes.rhai_eng.is_none() {
                    runtimes.rhai_eng = Some(rhai::Engine::new());
                    runtimes.rhai_eng.as_mut().unwrap().set_max_call_levels(15);
                }
                let script_untyped = registry.get_untyped(script.script_id.as_ref().unwrap().id());
                let script_typed = script_untyped.unwrap().get::<Script>(registry).unwrap();
                let source_payload = std::str::from_utf8(&script_typed.compiled_script).unwrap();
                println!("{}", &source_payload);

                let ast = runtimes
                    .rhai_eng
                    .as_ref()
                    .unwrap()
                    .compile(source_payload)
                    .unwrap();
                runtimes
                    .rhai_asts
                    .push((script.clone(), Rc::new(RefCell::new(ast))));
            }
        } else {
            for runtime in &runtimes.rhai_asts {
                let mut scope = Scope::new();
                let arg = i64::from_str(runtime.0.input_values[0].as_str()).unwrap();
                let result: i64 = runtimes
                    .rhai_eng
                    .as_ref()
                    .unwrap()
                    .call_fn(
                        &mut scope,
                        &runtime.1.borrow(),
                        runtime.0.entry_fn.as_str(),
                        (arg,),
                    )
                    .unwrap();
                println!("Rhai: fibonacci({}) = {}", &arg, result);
            }
        }
    }

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

pub struct ScriptingEventCache {
    mouse_motion: MouseMotion,
}

impl Default for ScriptingEventCache {
    fn default() -> Self {
        Self {
            mouse_motion: MouseMotion {
                delta: Vec2::default(),
            },
        }
    }
}

#[derive(Component)]
struct RuneScriptExecutionComponent {
    vm_index: usize,
    entry_fn: rune::Hash,
    input_args: Vec<String>,
}

#[derive(Default)]
struct RuneVMCollection {
    vms: Vec<Option<rune::Vm>>,
}

impl RuneVMCollection {
    fn append_new_vm(&mut self, context: &rune::Context, unit: rune::Unit) -> usize {
        let vm = rune::Vm::new(Arc::new(context.runtime()), Arc::new(unit));
        self.vms.push(Some(vm));
        self.vms.len() - 1
    }

    fn get_mut(&mut self, index: usize) -> &mut Option<rune::Vm> {
        &mut self.vms[index]
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
