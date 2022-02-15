use std::{
    str::{self, FromStr},
    sync::Arc,
};

use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::prelude::*;
use lgn_tracing::prelude::*;
use lgn_transform::prelude::*;
use rune::{
    runtime::Protocol,
    termcolor::{ColorChoice, StandardStream},
    Any, Context, ContextError, Diagnostics, Hash, Module, Source, Sources, ToValue, Unit, Value,
    Vm,
};

use crate::{
    runtime::{Script, ScriptComponent},
    ScriptType, ScriptingEventCache, ScriptingStage,
};

pub(crate) fn build(app: &mut App) -> Result<(), ContextError> {
    let mut context = rune_modules::default_context()?;

    context.install(&make_math_module()?)?;

    app.init_non_send_resource::<VMCollection>()
        .insert_resource(context)
        .add_system_to_stage(ScriptingStage::Compile, compile)
        .add_system_to_stage(ScriptingStage::Execute, tick.exclusive_system());

    Ok(())
}

fn compile(
    scripts: Query<'_, '_, (Entity, &ScriptComponent)>,
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

    drop(scripts);
    drop(rune_context);
    drop(registry);
}

fn tick(world: &mut World) {
    // get all entities with a compiled Rune script
    let scripted_entities = world
        .query_filtered::<Entity, With<ScriptExecutionContext>>()
        .iter(world)
        .collect::<Vec<_>>();

    let event_cache = world
        .get_resource::<ScriptingEventCache>()
        .cloned()
        .unwrap();

    for entity in scripted_entities {
        let mut args: Vec<Value> = Vec::new();

        let (script_vm_index, script_entry_fn) = {
            let mut entity = world.entity_mut(entity);
            let script = entity.get::<ScriptExecutionContext>().unwrap();

            for input in &script.input_args {
                if input == "mouse_motion.delta" {
                    let delta = Vec2 {
                        0: event_cache.mouse_motion.delta,
                    };
                    args.push(delta.to_value().unwrap());
                } else if input == "self.transform.translation" {
                    let transform = entity.get_mut::<Transform>();
                    if let Some(transform) = transform {
                        let translation = Vec3 {
                            0: transform.translation,
                        };
                        args.push(translation.to_value().unwrap());
                    }
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
            let vm = rune_vms.get_mut(script_vm_index).as_mut().unwrap();
            let _result = vm.execute(script_entry_fn, args).unwrap();
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

// Rune wrappers for standard types

#[derive(Any)]
struct Vec2(lgn_math::Vec2);

#[derive(Any)]
struct Vec3(lgn_math::Vec3);

fn make_math_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_math");

    // Vec2
    module.ty::<Vec2>()?;
    // Vec2.x
    module.field_fn(Protocol::GET, "x", |v: &Vec2| v.0.x)?;
    module.field_fn(Protocol::SET, "x", |v: &mut Vec2, x: f32| {
        v.0.x = x;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "x", |v: &mut Vec2, x: f32| {
        v.0.x += x;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "x", |v: &mut Vec2, x: f32| {
        v.0.x -= x;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "x", |v: &mut Vec2, x: f32| {
        v.0.x *= x;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "x", |v: &mut Vec2, x: f32| {
        v.0.x /= x;
    })?;
    // Vec2.y
    module.field_fn(Protocol::GET, "y", |v: &Vec2| v.0.y)?;
    module.field_fn(Protocol::SET, "y", |v: &mut Vec2, y: f32| {
        v.0.y = y;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "y", |v: &mut Vec2, y: f32| {
        v.0.y += y;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "y", |v: &mut Vec2, y: f32| {
        v.0.y -= y;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "y", |v: &mut Vec2, y: f32| {
        v.0.y *= y;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "y", |v: &mut Vec2, y: f32| {
        v.0.y /= y;
    })?;

    // Vec3
    module.ty::<Vec3>()?;
    // Vec3.x
    module.field_fn(Protocol::GET, "x", |v: &Vec3| v.0.x)?;
    module.field_fn(Protocol::SET, "x", |v: &mut Vec3, x: f32| {
        v.0.x = x;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.0.x += x;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.0.x -= x;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.0.x *= x;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.0.x /= x;
    })?;
    // Vec3.y
    module.field_fn(Protocol::GET, "y", |v: &Vec3| v.0.y)?;
    module.field_fn(Protocol::SET, "y", |v: &mut Vec3, y: f32| {
        v.0.y = y;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.0.y += y;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.0.y -= y;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.0.y *= y;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.0.y /= y;
    })?;
    // Vec3.z
    module.field_fn(Protocol::GET, "z", |v: &Vec3| v.0.z)?;
    module.field_fn(Protocol::SET, "z", |v: &mut Vec3, z: f32| {
        v.0.z = z;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.0.z += z;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.0.z -= z;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.0.z *= z;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.0.z /= z;
    })?;

    Ok(module)
}
