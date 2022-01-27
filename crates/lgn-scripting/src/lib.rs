//! Scripting library - currently using the MUN language

// generated from def\script.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod plugin;
pub use plugin::*;

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

/*use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use components::ECSScriptType;
use components::{ECSScriptComponent, ECSScriptPayload};
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use rune::termcolor::ColorChoice;
use rune::termcolor::StandardStream;
use rune::FromValue;
use rune::ToValue;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Default)]
struct RuntimeScripts {
    pub mun_runtimes: Vec<(ECSScriptComponent, Rc<RefCell<mun_runtime::Runtime>>)>,
    pub rune_vm: Option<rune::Vm>,
    pub rhai_eng: Option<rhai::Engine>,
    pub rhai_asts: Vec<(ECSScriptComponent, Rc<RefCell<rhai::AST>>)>,
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
        let mun_components = scripts
            .iter()
            .filter(|s| s.script_type == ECSScriptType::Mun);
        let rune_components = scripts
            .iter()
            .filter(|s| s.script_type == ECSScriptType::Rune);
        let rhai_components = scripts
            .iter()
            .filter(|s| s.script_type == ECSScriptType::Rhai);

        Self::tick_mun(mun_components, &mut runtimes);
        Self::tick_rune(rune_components, &mut runtimes);
        Self::tick_rhai(rhai_components, &mut runtimes);
    }

    fn tick_mun<'a>(
        mun_components: impl Iterator<Item = &'a ECSScriptComponent>,
        runtimes: &mut RuntimeScripts,
    ) {
        if runtimes.mun_runtimes.is_empty() {
            for script in mun_components {
                let lib_path = match &script.payload {
                    ECSScriptPayload::LibPath(path) => path,
                    _ => panic!("Unexpected script payload!"),
                };
                let runtime = mun_runtime::RuntimeBuilder::new(&lib_path)
                    .spawn()
                    .expect("Failed to spawn Runtime");
                runtimes.mun_runtimes.push((script.clone(), runtime));
            }
        }
        for runtime in &runtimes.mun_runtimes {
            {
                let runtime_ref = runtime.1.deref().borrow();
                let result: i64 = mun_runtime::invoke_fn!(
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

    fn tick_rune<'a>(
        rune_components: impl Iterator<Item = &'a ECSScriptComponent>,
        runtimes: &mut RuntimeScripts,
    ) {
        if runtimes.rune_vm.is_none() {
            for script in rune_components {
                let context = rune_modules::default_context().unwrap();

                let source_payload = match &script.payload {
                    ECSScriptPayload::ContainedScript(text) => text,
                    _ => panic!("Unexpected script payload!"),
                };
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
                let fn_name = &["fib"];
                let hashed_fn_name = rune::Hash::type_hash(fn_name);

                let output = runtimes
                    .rune_vm
                    .as_mut()
                    .unwrap()
                    .execute(hashed_fn_name, args)
                    .unwrap()
                    .complete()
                    .unwrap();
                let output = i64::from_value(output).unwrap();

                println!("output: {}", output);
            }
        }
    }

    fn tick_rhai<'a>(
        rhai_components: impl Iterator<Item = &'a ECSScriptComponent>,
        runtimes: &mut RuntimeScripts,
    ) {
        if runtimes.rhai_eng.is_none() {
            for script in rhai_components {
                if runtimes.rhai_eng.is_none() {
                    runtimes.rhai_eng = Some(rhai::Engine::new());
                }
                let script_source = match &script.payload {
                    ECSScriptPayload::ContainedScript(text) => text,
                    _ => panic!("Bad script payload!"),
                };
                let ast = runtimes
                    .rhai_eng
                    .as_ref()
                    .unwrap()
                    .compile(script_source)
                    .unwrap();
                runtimes
                    .rhai_asts
                    .push((script.clone(), Rc::new(RefCell::new(ast))));
            }
        } else {
            for runtime in &runtimes.rhai_asts {
                let output = runtimes
                    .rhai_eng
                    .as_ref()
                    .unwrap()
                    .eval_ast::<i64>(&runtime.1.borrow_mut())
                    .unwrap();
                println!("output: {}", output);
            }
        }
    }
}*/