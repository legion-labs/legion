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

(to do)

## DCES

(to do)

## Dotrix

(to do)

## ecs-rs

(to do)

## hecs

(to do)

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

pub trait IntoWorkloadSystem<B, R> {
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
