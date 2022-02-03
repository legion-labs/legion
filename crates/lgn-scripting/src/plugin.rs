use lgn_app::prelude::*;
#[cfg(feature = "offline")]
use lgn_data_offline::resource::ResourceRegistryOptions;
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};
use lgn_ecs::prelude::*;
use rhai::Scope;
use rune::{
    termcolor::{ColorChoice, StandardStream},
    FromValue, ToValue,
};

use std::{cell::RefCell, rc::Rc, sync::Arc, fs};

use crate::runtime::{Script, ScriptComponent};
use std::str::FromStr;

#[derive(Default)]
struct RuntimeScripts {
    pub mun_runtimes: Vec<(ScriptComponent, Rc<RefCell<mun_runtime::Runtime>>)>,
    pub rune_vm: Option<rune::Vm>,
    pub rhai_eng: Option<rhai::Engine>,
    pub rhai_asts: Vec<(ScriptComponent, Rc<RefCell<rhai::AST>>)>,
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
        registry: Res<'_, Arc<AssetRegistry>>,
    ) {
        let mun_components = scripts
            .iter()
            .filter(|s| s.script_type == 1 /*ScriptType::Mun*/);
        let rune_components = scripts
            .iter()
            .filter(|s| s.script_type == 2 /*ScriptType::Rune*/);
        let rhai_components = scripts
            .iter()
            .filter(|s| s.script_type == 3 /*ScriptType::Rhai*/);

        let r = runtimes.into_inner();
        Self::tick_mun(mun_components, r, &registry);
        Self::tick_rune(rune_components, r, &registry);
        Self::tick_rhai(rhai_components, r, &registry);
    }

    fn tick_mun<'a>(
        mun_components: impl Iterator<Item = &'a ScriptComponent>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
    ) {
        if runtimes.mun_runtimes.is_empty() {
            for script in mun_components {
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
                let result: i64 = mun_runtime::invoke_fn!(
                    runtime_ref,
                    &runtime.0.entry_fn,
                    arg
                )
                .unwrap();
                println!("Mun: fibonacci({}) = {}", &arg, result);
            }

            // reload the script of the path changed
            runtime.1.borrow_mut().update();
        }
    }

    fn tick_rune<'a>(
        rune_components: impl Iterator<Item = &'a ScriptComponent>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
    ) {
        if runtimes.rune_vm.is_none() {
            for script in rune_components {
                let context = rune_modules::default_context().unwrap();

                let script_untyped = registry.get_untyped(script.script_id.as_ref().unwrap().id());
                let script_typed = script_untyped.unwrap().get::<Script>(registry).unwrap();
                let source_payload = std::str::from_utf8(&script_typed.compiled_script).unwrap();
                println!("{}", &source_payload);

                let mut sources = rune::Sources::new();
                sources.insert(rune::Source::new("entry", &source_payload));

                let mut diagnostics = rune::Diagnostics::new();

                let result = rune::prepare(&mut sources)
                    .with_context(&context)
                    .with_diagnostics(&mut diagnostics)
                    .build();

                if !diagnostics.is_empty() {
                    let mut writer = StandardStream::stderr(ColorChoice::Always);
                    diagnostics.emit(&mut writer, &sources).unwrap();
                }

                let unit = result.unwrap();

                runtimes.rune_vm = Some(rune::Vm::new(Arc::new(context.runtime()), Arc::new(unit)));
            }
        } else {
            for script in rune_components {
                let arg = i64::from_str(&script.input_values[0]).unwrap();

                let args = vec![arg.to_value().unwrap()];
                let fn_name = &[script.entry_fn.as_str()];
                let hashed_fn_name = rune::Hash::type_hash(fn_name);

                let result = runtimes
                    .rune_vm
                    .as_mut()
                    .unwrap()
                    .execute(hashed_fn_name, args)
                    .unwrap()
                    .complete()
                    .unwrap();
                let result = i64::from_value(result).unwrap();
                println!("Rune: fibonacci({}) = {}", &arg, result);
            }
        }
    }

    fn tick_rhai<'a>(
        rhai_components: impl Iterator<Item = &'a ScriptComponent>,
        runtimes: &mut RuntimeScripts,
        registry: &AssetRegistry,
    ) {
        if runtimes.rhai_eng.is_none() {
            for script in rhai_components {
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
