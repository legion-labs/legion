#![allow(unsafe_code)]

use std::{
    fmt::{self, Write},
    str::{self, FromStr},
    sync::Arc,
};

use lgn_app::prelude::*;
use lgn_core::prelude::*;
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
                    let events = Events(event_cache);
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

// Rune wrappers for standard types

#[derive(Any)]
struct Events(*const ScriptingEventCache);

fn make_scripting_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_scripting");

    module.ty::<Events>()?;
    module.field_fn(Protocol::GET, "mouse_motion", |events: &Events| unsafe {
        Vec2::new(&(*events.0).mouse_motion.delta)
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

#[derive(Any)]
struct EntityLookupByName {
    world: *mut World,
}

impl EntityLookupByName {
    fn new(world: *mut World) -> Self {
        Self { world }
    }

    fn lookup(&self, entity_name: &str) -> Option<Entity> {
        let world = unsafe { &mut *self.world };

        let mut query = world.query::<(lgn_ecs::prelude::Entity, &Name)>();
        let entity_name: Name = entity_name.into();

        for (entity, name) in query.iter(world) {
            if entity_name == *name {
                return Some(Entity::new(self.world, entity));
            }
        }

        None
    }
}

fn make_ecs_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_ecs");

    module.ty::<Entity>()?;
    module.field_fn(Protocol::GET, "transform", |entity: &Entity| {
        Transform::new(entity)
    })?;

    module.ty::<EntityLookupByName>()?;
    module.inst_fn(Protocol::INDEX_GET, EntityLookupByName::lookup)?;

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
        |transform: &Transform| unsafe { Vec3::new(&mut (*transform.0).translation) },
    )?;

    Ok(module)
}

#[derive(Any)]
struct Vec2(*const lgn_math::Vec2);

impl Vec2 {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn new(vec: &lgn_math::Vec2) -> Self {
        Self(vec as *const lgn_math::Vec2)
    }

    fn get(&self) -> &lgn_math::Vec2 {
        unsafe { &*self.0 }
    }

    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.get())
    }
}

fn normalize2(x: f32, y: f32) -> (f32, f32) {
    let vec = lgn_math::Vec2::new(x, y);
    let vec = vec.normalize_or_zero();
    (vec.x, vec.y)
}

#[derive(Any)]
struct Vec3(*mut lgn_math::Vec3);

impl Vec3 {
    fn new(vec: &mut lgn_math::Vec3) -> Self {
        Self(vec as *mut lgn_math::Vec3)
    }

    fn get(&self) -> &lgn_math::Vec3 {
        unsafe { &*self.0 }
    }

    fn get_mut(&mut self) -> &mut lgn_math::Vec3 {
        unsafe { &mut *self.0 }
    }

    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.get())
    }

    fn clamp_x(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.x = v.x.clamp(min, max);
    }

    fn clamp_y(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.y = v.y.clamp(min, max);
    }

    fn clamp_z(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.z = v.z.clamp(min, max);
    }
}

fn random() -> f32 {
    rand::random::<f32>()
}

fn make_math_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_math");

    module.function(&["random"], random)?;
    module.function(&["normalize2"], normalize2)?;

    module.ty::<Vec2>()?;
    module.inst_fn(Protocol::STRING_DISPLAY, Vec2::display)?;
    module.field_fn(Protocol::GET, "x", |v: &Vec2| v.get().x)?;
    module.field_fn(Protocol::GET, "y", |v: &Vec2| v.get().y)?;

    module.ty::<Vec3>()?;
    module.inst_fn(Protocol::STRING_DISPLAY, Vec3::display)?;
    module.inst_fn("clamp_x", Vec3::clamp_x)?;
    module.inst_fn("clamp_y", Vec3::clamp_y)?;
    module.inst_fn("clamp_z", Vec3::clamp_z)?;
    module.field_fn(Protocol::GET, "x", |v: &Vec3| v.get().x)?;
    module.field_fn(Protocol::SET, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x = x;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x += x;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x -= x;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x *= x;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x /= x;
    })?;
    module.field_fn(Protocol::GET, "y", |v: &Vec3| v.get().y)?;
    module.field_fn(Protocol::SET, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y = y;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y += y;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y -= y;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y *= y;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y /= y;
    })?;
    module.field_fn(Protocol::GET, "z", |v: &Vec3| v.get().z)?;
    module.field_fn(Protocol::SET, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z = z;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z += z;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z -= z;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z *= z;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z /= z;
    })?;

    Ok(module)
}
