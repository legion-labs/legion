use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};

use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::prelude::*;
use rhai::Scope;

use crate::runtime::{Script, ScriptComponent};

pub(crate) fn build(app: &mut App) {
    let mut rhai_eng = rhai::Engine::new();
    rhai_eng.set_max_call_levels(15);

    app.insert_non_send_resource(rhai_eng)
        .init_non_send_resource::<ASTCollection>()
        .add_system(compile)
        .add_system(tick);
}

fn compile(
    scripts: Query<'_, '_, (Entity, &mut ScriptComponent)>,
    rhai_eng: NonSend<'_, rhai::Engine>,
    mut ast_collection: NonSendMut<'_, ASTCollection>,
    registry: Res<'_, Arc<AssetRegistry>>,
    mut commands: Commands<'_, '_>,
) {
    let rhai_scripts = scripts
        .iter()
        .filter(|(_entity, s)| s.script_type == 3 /*ScriptType::Rhai*/);

    for (entity, script) in rhai_scripts {
        let script_untyped = registry.get_untyped(script.script_id.as_ref().unwrap().id());
        let script_typed = script_untyped.unwrap().get::<Script>(&registry).unwrap();
        let source_payload = std::str::from_utf8(&script_typed.compiled_script).unwrap();
        println!("{}", &source_payload);

        let ast = rhai_eng.compile(source_payload).unwrap();

        let script_exec = ScriptExecutionContext {
            ast_index: ast_collection.append(ast),
            entry_fn: script.entry_fn.clone(),
            input_values: script.input_values.clone(),
        };

        commands
            .entity(entity)
            .insert(script_exec)
            .remove::<ScriptComponent>();
    }

    drop(scripts);
    drop(rhai_eng);
    drop(registry);
}

fn tick(
    query: Query<'_, '_, (Entity, &mut ScriptExecutionContext)>,
    rhai_eng: NonSend<'_, rhai::Engine>,
    ast_collection: NonSend<'_, ASTCollection>,
) {
    for (_entity, script) in query.iter() {
        let mut scope = Scope::new();
        let arg = i64::from_str(script.input_values[0].as_str()).unwrap();
        let result: i64 = rhai_eng
            .call_fn(
                &mut scope,
                &ast_collection.get(script.ast_index).borrow(),
                script.entry_fn.as_str(),
                (arg,),
            )
            .unwrap();
        println!("Rhai: fibonacci({}) = {}", &arg, result);
    }

    drop(query);
    drop(rhai_eng);
    drop(ast_collection);
}

#[derive(Component)]
struct ScriptExecutionContext {
    ast_index: usize,
    entry_fn: String,
    input_values: Vec<String>,
}

#[derive(Default)]
struct ASTCollection {
    asts: Vec<Rc<RefCell<rhai::AST>>>,
}

impl ASTCollection {
    fn append(&mut self, ast: rhai::AST) -> usize {
        self.asts.push(Rc::new(RefCell::new(ast)));
        self.asts.len() - 1
    }

    fn get(&self, index: usize) -> &Rc<RefCell<rhai::AST>> {
        &self.asts[index]
    }
}
