#![allow(unsafe_code)]

use std::{
    str::{self, FromStr},
    sync::Arc,
};

use lgn_app::prelude::*;
use lgn_data_runtime::AssetRegistry;
use lgn_ecs::{prelude::*, world::EntityMut};
use lgn_tracing::prelude::*;
use rune::{
    runtime::Protocol,
    termcolor::{ColorChoice, StandardStream},
    Any, Context, ContextError, Diagnostics, Hash, Module, Source, Sources, ToValue, Unit, Value,
    Vm,
};

use crate::{
    plugin::get_script_payload, runtime::ScriptComponent, ScriptType, ScriptingEventCache,
    ScriptingStage,
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
        let script_payload = get_script_payload(script, &registry);
        let script_payload = str::from_utf8(script_payload).unwrap();
        info!("script payload: {}", &script_payload);

        let mut sources = Sources::new();
        sources.insert(Source::new("entry", &script_payload));

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
                    let events = Events(event_cache);
                    args.push(events.to_value().unwrap());
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
            let mut vm_exec = vm.execute(script_entry_fn, args).unwrap();
            let _result = vm_exec.complete().unwrap();
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
struct Events(*const ScriptingEventCache);

fn make_scripting_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_scripting");

    module.ty::<Events>()?;
    module.field_fn(Protocol::GET, "mouse_motion", |events: &Events| unsafe {
        Vec2Ref::new(&(*events.0).mouse_motion.delta)
    })?;

    Ok(module)
}

#[derive(Any)]
struct Entity {
    world: *mut World,
    entity: lgn_ecs::prelude::Entity,
}

impl Entity {
    fn new(world: *mut World, entity: lgn_ecs::prelude::Entity) -> Self {
        Self { world, entity }
    }

    fn get_mut(&self) -> EntityMut<'_> {
        let world = unsafe { &mut *self.world };
        world.entity_mut(self.entity)
    }
}

fn make_ecs_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_ecs");

    module.ty::<Entity>()?;
    module.field_fn(Protocol::GET, "transform", |entity: &Entity| {
        Transform::new(entity)
    })?;

    Ok(module)
}

#[derive(Any)]
struct Transform(*mut lgn_transform::prelude::Transform);

impl Transform {
    fn new(entity: &Entity) -> Self {
        let mut entity = entity.get_mut();
        let transform = entity
            .get_mut::<lgn_transform::prelude::Transform>()
            .unwrap()
            .into_inner();
        Self(transform as *mut lgn_transform::prelude::Transform)
    }
}

fn make_transform_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_transform");

    module.ty::<Transform>()?;
    module.field_fn(
        Protocol::GET,
        "translation",
        |transform: &Transform| unsafe { Vec3Mut::new(&mut (*transform.0).translation) },
    )?;

    Ok(module)
}

#[derive(Any)]
struct Vec2Ref(*const lgn_math::Vec2);

impl Vec2Ref {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn new(vec: &lgn_math::Vec2) -> Self {
        Self(vec as *const lgn_math::Vec2)
    }
}

// #[derive(Any)]
// struct Vec2Mut(*mut lgn_math::Vec2);

// impl Vec2Mut {
//     fn new(vec: &mut lgn_math::Vec2) -> Self {
//         Self(vec as *mut lgn_math::Vec2)
//     }
// }

// #[derive(Any)]
// struct Vec3Ref(*const lgn_math::Vec3);

// impl Vec3Ref {
//     fn new(vec: &lgn_math::Vec3) -> Self {
//         Self(vec as *const lgn_math::Vec3)
//     }
// }

#[derive(Any)]
struct Vec3Mut(*mut lgn_math::Vec3);

impl Vec3Mut {
    fn new(vec: &mut lgn_math::Vec3) -> Self {
        Self(vec as *mut lgn_math::Vec3)
    }
}

fn make_math_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_math");

    module.ty::<Vec2Ref>()?;
    module.field_fn(Protocol::GET, "x", |v: &Vec2Ref| unsafe { (*v.0).x })?;
    module.field_fn(Protocol::GET, "y", |v: &Vec2Ref| unsafe { (*v.0).y })?;

    // module.ty::<Vec2Mut>()?;
    // module.field_fn(Protocol::GET, "x", |v: &Vec2Mut| unsafe { (*v.0).x })?;
    // module.field_fn(Protocol::SET, "x", |v: &mut Vec2Mut, x: f32| unsafe {
    //     (*v.0).x = x;
    // })?;
    // module.field_fn(
    //     Protocol::ADD_ASSIGN,
    //     "x",
    //     |v: &mut Vec2Mut, x: f32| unsafe {
    //         (*v.0).x += x;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::SUB_ASSIGN,
    //     "x",
    //     |v: &mut Vec2Mut, x: f32| unsafe {
    //         (*v.0).x -= x;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::MUL_ASSIGN,
    //     "x",
    //     |v: &mut Vec2Mut, x: f32| unsafe {
    //         (*v.0).x *= x;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::DIV_ASSIGN,
    //     "x",
    //     |v: &mut Vec2Mut, x: f32| unsafe {
    //         (*v.0).x /= x;
    //     },
    // )?;
    // module.field_fn(Protocol::GET, "y", |v: &Vec2Mut| unsafe { (*v.0).y })?;
    // module.field_fn(Protocol::SET, "y", |v: &mut Vec2Mut, y: f32| unsafe {
    //     (*v.0).y = y;
    // })?;
    // module.field_fn(
    //     Protocol::ADD_ASSIGN,
    //     "y",
    //     |v: &mut Vec2Mut, y: f32| unsafe {
    //         (*v.0).y += y;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::SUB_ASSIGN,
    //     "y",
    //     |v: &mut Vec2Mut, y: f32| unsafe {
    //         (*v.0).y -= y;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::MUL_ASSIGN,
    //     "y",
    //     |v: &mut Vec2Mut, y: f32| unsafe {
    //         (*v.0).y *= y;
    //     },
    // )?;
    // module.field_fn(
    //     Protocol::DIV_ASSIGN,
    //     "y",
    //     |v: &mut Vec2Mut, y: f32| unsafe {
    //         (*v.0).y /= y;
    //     },
    // )?;

    // module.ty::<Vec3Ref>()?;
    // module.field_fn(Protocol::GET, "x", |v: &Vec3Ref| unsafe { (*v.0).x })?;
    // module.field_fn(Protocol::GET, "y", |v: &Vec3Ref| unsafe { (*v.0).y })?;
    // module.field_fn(Protocol::GET, "z", |v: &Vec3Ref| unsafe { (*v.0).z })?;

    module.ty::<Vec3Mut>()?;
    module.field_fn(Protocol::GET, "x", |v: &Vec3Mut| unsafe { (*v.0).x })?;
    module.field_fn(Protocol::SET, "x", |v: &mut Vec3Mut, x: f32| unsafe {
        (*v.0).x = x;
    })?;
    module.field_fn(
        Protocol::ADD_ASSIGN,
        "x",
        |v: &mut Vec3Mut, x: f32| unsafe {
            (*v.0).x += x;
        },
    )?;
    module.field_fn(
        Protocol::SUB_ASSIGN,
        "x",
        |v: &mut Vec3Mut, x: f32| unsafe {
            (*v.0).x -= x;
        },
    )?;
    module.field_fn(
        Protocol::MUL_ASSIGN,
        "x",
        |v: &mut Vec3Mut, x: f32| unsafe {
            (*v.0).x *= x;
        },
    )?;
    module.field_fn(
        Protocol::DIV_ASSIGN,
        "x",
        |v: &mut Vec3Mut, x: f32| unsafe {
            (*v.0).x /= x;
        },
    )?;
    module.field_fn(Protocol::GET, "y", |v: &Vec3Mut| unsafe { (*v.0).y })?;
    module.field_fn(Protocol::SET, "y", |v: &mut Vec3Mut, y: f32| unsafe {
        (*v.0).y = y;
    })?;
    module.field_fn(
        Protocol::ADD_ASSIGN,
        "y",
        |v: &mut Vec3Mut, y: f32| unsafe {
            (*v.0).y += y;
        },
    )?;
    module.field_fn(
        Protocol::SUB_ASSIGN,
        "y",
        |v: &mut Vec3Mut, y: f32| unsafe {
            (*v.0).y -= y;
        },
    )?;
    module.field_fn(
        Protocol::MUL_ASSIGN,
        "y",
        |v: &mut Vec3Mut, y: f32| unsafe {
            (*v.0).y *= y;
        },
    )?;
    module.field_fn(
        Protocol::DIV_ASSIGN,
        "y",
        |v: &mut Vec3Mut, y: f32| unsafe {
            (*v.0).y /= y;
        },
    )?;
    module.field_fn(Protocol::GET, "z", |v: &Vec3Mut| unsafe { (*v.0).z })?;
    module.field_fn(Protocol::SET, "z", |v: &mut Vec3Mut, z: f32| unsafe {
        (*v.0).z = z;
    })?;
    module.field_fn(
        Protocol::ADD_ASSIGN,
        "z",
        |v: &mut Vec3Mut, z: f32| unsafe {
            (*v.0).z += z;
        },
    )?;
    module.field_fn(
        Protocol::SUB_ASSIGN,
        "z",
        |v: &mut Vec3Mut, z: f32| unsafe {
            (*v.0).z -= z;
        },
    )?;
    module.field_fn(
        Protocol::MUL_ASSIGN,
        "z",
        |v: &mut Vec3Mut, z: f32| unsafe {
            (*v.0).z *= z;
        },
    )?;
    module.field_fn(
        Protocol::DIV_ASSIGN,
        "z",
        |v: &mut Vec3Mut, z: f32| unsafe {
            (*v.0).z /= z;
        },
    )?;

    Ok(module)
}
