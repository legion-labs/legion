# Entity Component Systems - Research - Systems

How are systems implemented in existing Rust ECS solutions?

Solutions:
* [Bevy](#Bevy)
* [DCES](#DCES)
* [Dotrix](#Dotrix)
* [ecs-rs](#ecs-rs)
* [hecs](#hecs)
* [Legion](#Legion)
* [Planck](#Planck)
* [Rustic](#Rustic)
* [Shipyard](#Shipyard)
* [Specs](#Specs)

## Bevy

Initial Bevy ECS was based on hecs, but has been rewritten.

[Redesign notes](https://bevyengine.org/news/bevy-0-5/#bevy-ecs-v2), and associated [pull request](https://github.com/bevyengine/bevy/pull/1525).

### Usage

```Rust
// systems access components by using queries
fn score_system(mut query: Query<(&Player, &mut Score)>) { ... }

// systems can access global resources
fn new_round_system(game_rules: Res<GameRules>, mut game_state: ResMut<GameState>) { ... }

// can mix and match
fn score_check_system(game_rules: Res<GameRules>,
    mut game_state: ResMut<GameState>,
    query: Query<(&Player, &Score)>,
) { ... }

// registration
App::new()
    .add_startup_system(startup_system)
    .add_system(print_message_system)
```

### Storage

``` Rust
// can register anything that implements the IntoSystemDescriptor trait

trait IntoSystemDescriptor<Params> {
    fn into_descriptor(self) -> SystemDescriptor;
}

enum SystemDescriptor {
    Parallel(ParallelSystemDescriptor),
    Exclusive(ExclusiveSystemDescriptor), // no input, has access to world only
}

struct ParallelSystemDescriptor {
    system: BoxedSystem<(), ()>,
    ...
}

type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;

pub trait System: Send + Sync + 'static {
    type In;
    type Out;
    ...
    unsafe fn run_unsafe(&mut self, input: Self::In, world: &World) -> Self::Out;
    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {
        unsafe { self.run_unsafe(input, world) }
    }
    ...
}

// converting function to System

impl<Params, S> IntoSystemDescriptor<Params> for S
where
    S: IntoSystem<(), (), Params>,
{
    fn into_descriptor(self) -> SystemDescriptor {
        new_parallel_descriptor(Box::new(self.system())).into_descriptor()
    }
}

trait IntoSystem<In, Out, Params> {
    type System: System<In = In, Out = Out>;
    fn system(self) -> Self::System;
}

struct FunctionSystem<In, Out, Param, Marker, F>
where
    Param: SystemParam,
{
    func: F,
    ...
}

impl<In, Out, Param, Marker, F> IntoSystem<In, Out, (IsFunctionSystem, Param, Marker)> for F
where
    ...
    F: SystemParamFunction<In, Out, Param, Marker> + Send + Sync + 'static,
{
    type System = FunctionSystem<In, Out, Param, Marker, F>;
    fn system(self) -> Self::System {
        FunctionSystem {
            func: self,
            ...
        }
    }
}

// all parameters must implement the SystemParam trait, which can be derived
pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a>;
}
```
The `impl_system_function!` macro (in combination with `all_tuples!`) is used to implement `SystemParamFunction` for all function types with arity up to 16. The implementations take care of packing/unpacking the argument list to a tuple. (See [`function_system.rs`](https://github.com/bevyengine/bevy/blob/main/crates/legion_ecs/src/system/function_system.rs) for code)

The function types match with this pattern:
```Rust
for <'a> &'a mut Func:
    FnMut(In<Input>, $($param),*) -> Out +
    FnMut(In<Input>, $(<<$param as SystemParam>::Fetch as SystemParamFetch>::Item),*) -> Out
```

### References

* Bevy the book, [section 2.3. ecs](https://bevyengine.org/learn/book/getting-started/ecs/)
* [ecs crate README](https://github.com/bevyengine/bevy/blob/main/crates/legion_ecs/README.md)
* [ECS guided introduction](https://github.com/bevyengine/bevy/blob/latest/examples/ecs/ecs_guide.rs)

## DCES

(to do)

## Dotrix

(to do)

## ecs-rs

(to do)

## hecs

In hecs, there are no explicit systems per se.

You can use queries directly on the world, and the scheduling / sequencing of system-like functions is up to the developer.

### Usage

```Rust
fn system_integrate_motion(world: &mut World, query: &mut PreparedQuery<(&mut Position, &Speed)>) {
    for (id, (pos, s)) in query.query_mut(world) {
        ...
    }
}

fn system_fire_at_closest(world: &mut World) {
    for (id0, (pos0, dmg0, kc0)) in
        &mut world.query::<With<Health, (&Position, &Damage, &mut KillCount)>>()
    {
        // Find closest:
        // Nested queries are O(n^2) and you usually want to avoid that by using some sort of
        // spatial index like a quadtree or more general BVH
        let closest = world
            .query::<With<Health, &Position>>()
            .iter()
            .filter(|(id1, _)| *id1 != id0)
            .min_by_key(|(_, pos1)| manhattan_dist(pos0.x, pos1.x, pos0.y, pos1.y))
            .map(|(entity, _pos)| entity);
        ...
        let mut hp1 = world.get_mut::<Health>(closest).unwrap();
        ...
    }
}

fn main() {
    let mut world = World::new();
    ...
    let mut motion_query = PreparedQuery::<(&mut Position, &Speed)>::default();

    loop {
        ...
        // Run all simulation systems:
        system_integrate_motion(&mut world, &mut motion_query);
        system_fire_at_closest(&mut world);
        ...
    }
}
```
### References
* [hecs README](https://github.com/Ralith/hecs)
## Legion

(to do)

## Planck

(to do)

## Rustic

Mostly just the Entity and Component part, no Systems... :confused:

## Shipyard

Alternative to Specs, inspired by EnTT (C++) which uses `SparseSet`.

### Usage

```Rust
fn in_acid(positions: View<Position>, mut healths: ViewMut<Health>) {
    for (_, mut health) in (&positions, &mut healths)
        .iter()
        .filter(|(pos, _)| is_in_acid(pos))
    {
        health.0 -= 1;
    }
}

fn main() {
    ...
    world.run(in_acid).unwrap();
}
```
We call `run_with_data` instead of run when we want to pass data to a system.

If you want to pass multiple variables, you can use a tuple.

### Storage

```Rust
pub struct WorkloadBuilder {
    pub(super) systems: Vec<WorkUnit>,
    ...
}

pub(super) enum WorkUnit {
    System(WorkloadSystem),
    ...
}

pub trait IntoWorkloadSystem<B, R> for F
where
    F: 'static + Send + Sync + Fn() -> R,
{
    fn into_workload_system(self) -> Result<WorkloadSystem, error::InvalidSystem> {
        Ok(WorkloadSystem {
            system_fn: Box::new(move |_: &World| {
                (self)();
                Ok(())
            }),
            ...
        })
    }
}

pub struct WorkloadSystem {
    pub(super) system_fn: Box<dyn Fn(&World) -> Result<(), error::Run> + Send + Sync + 'static>,
    ...
}

```

Macros that allow conversion of types for multiple arguments in [`into_workload_system.rs`](https://github.com/leudz/shipyard/blob/master/src/scheduler/into_workload_system.rs)

### References

* [Shipyard User's Guide](https://leudz.github.io/shipyard/guide/0.5.0)
    * [Systems](https://leudz.github.io/shipyard/guide/0.5.0/fundamentals/systems.html)

## Specs

Part of the Amethyst project, Specs is its ECS system. The actual system scheduling portion is in the Amethyst [shred](https://crates.io/crates/shred) crate.

`System` is a trait, with an associated type named `SystemData`.

```Rust
pub trait System<'a> {
    type SystemData: DynamicSystemData<'a>;
    
    fn run(&mut self, data: Self::SystemData);

    // Return the accessor from the [`SystemData`].
    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> { ... }
}
```

### Usage

```Rust
struct SysA;

impl<'a> System<'a> for SysA {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);
    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
        }
    }
}
```

### Storage

Systems are stored in the dispatcher, which uses stages:
```Rust
pub struct Stage<'a> {
    groups: GroupVec<ArrayVec<[SystemExecSend<'a>; MAX_SYSTEMS_PER_GROUP]>>,
}

pub type SystemExecSend<'b> = Box<dyn for<'a> RunNow<'a> + Send + 'b>;

pub trait RunNow<'a> {
    fn run_now(&mut self, world: &'a World);
    ...
}

impl<'a, T> RunNow<'a> for T
where
    T: System<'a>,
{
    fn run_now(&mut self, world: &'a World) {
        let data = T::SystemData::fetch(&self.accessor(), world);
        self.run(data);
    }
    ...
}
```

You will mostly use a tuple of system data (which also implements `SystemData`). You can also create such a resource bundle by simply deriving `SystemData` for a struct.

The macro `impl_data` is used to implement SystemData for tuples of sizes from 1 to 26. See [`system.rs`](https://github.com/amethyst/shred/blob/master/src/system.rs).

### References

* [System Data](https://specs.amethyst.rs/docs/tutorials/06_system_data.html), The Specs Book
