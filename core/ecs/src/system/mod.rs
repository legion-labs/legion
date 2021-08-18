mod commands;
mod exclusive_system;
mod function_system;
mod query;
#[allow(clippy::module_inception)]
mod system;
mod system_chaining;
mod system_param;

pub use commands::*;
pub use exclusive_system::*;
pub use function_system::*;
pub use query::*;
pub use system::*;
pub use system_chaining::*;
pub use system_param::*;

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use std::any::TypeId;

    use crate::{
        archetype::Archetypes,
        bundle::Bundles,
        component::Components,
        entity::{Entities, Entity},
        query::{Added, Changed, Or, With, Without},
        schedule::{Schedule, Stage, SystemStage},
        system::{
            ConfigurableSystem, IntoExclusiveSystem, IntoSystem, Local, Query, QuerySet,
            RemovedComponents, Res, ResMut, System, SystemState,
        },
        world::{FromWorld, World},
    };

    #[derive(Debug, Eq, PartialEq, Default)]
    struct A;
    struct B;
    struct C;
    struct D;
    struct E;
    struct F;

    #[test]
    fn simple_system() {
        fn sys(query: Query<'_, &A>) {
            for a in query.iter() {
                println!("{:?}", a);
            }
        }

        let mut system = sys.system();
        let mut world = World::new();
        world.spawn().insert(A);

        system.initialize(&mut world);
        for archetype in world.archetypes.iter() {
            system.new_archetype(archetype);
        }
        system.run((), &mut world);
    }

    fn run_system<Param, S: IntoSystem<(), (), Param>>(world: &mut World, system: S) {
        let mut schedule = Schedule::default();
        let mut update = SystemStage::parallel();
        update.add_system(system);
        schedule.add_stage("update", update);
        schedule.run(world);
    }

    #[test]
    fn query_system_gets() {
        fn query_system(
            mut ran: ResMut<'_, bool>,
            entity_query: Query<'_, Entity, With<A>>,
            b_query: Query<'_, &B>,
            a_c_query: Query<'_, (&A, &C)>,
            d_query: Query<'_, &D>,
        ) {
            let entities = entity_query.iter().collect::<Vec<Entity>>();
            assert!(
                b_query.get_component::<B>(entities[0]).is_err(),
                "entity 0 should not have B"
            );
            assert!(
                b_query.get_component::<B>(entities[1]).is_ok(),
                "entity 1 should have B"
            );
            assert!(
                b_query.get_component::<A>(entities[1]).is_err(),
                "entity 1 should have A, but b_query shouldn't have access to it"
            );
            assert!(
                b_query.get_component::<D>(entities[3]).is_err(),
                "entity 3 should have D, but it shouldn't be accessible from b_query"
            );
            assert!(
                b_query.get_component::<C>(entities[2]).is_err(),
                "entity 2 has C, but it shouldn't be accessible from b_query"
            );
            assert!(
                a_c_query.get_component::<C>(entities[2]).is_ok(),
                "entity 2 has C, and it should be accessible from a_c_query"
            );
            assert!(
                a_c_query.get_component::<D>(entities[3]).is_err(),
                "entity 3 should have D, but it shouldn't be accessible from b_query"
            );
            assert!(
                d_query.get_component::<D>(entities[3]).is_ok(),
                "entity 3 should have D"
            );

            *ran = true;
        }

        let mut world = World::default();
        world.insert_resource(false);
        world.spawn().insert_bundle((A,));
        world.spawn().insert_bundle((A, B));
        world.spawn().insert_bundle((A, C));
        world.spawn().insert_bundle((A, D));

        run_system(&mut world, query_system);

        assert!(*world.get_resource::<bool>().unwrap(), "system ran");
    }

    #[test]
    fn or_query_set_system() {
        // Regression test for issue #762
        fn query_system(
            mut ran: ResMut<'_, bool>,
            set: QuerySet<(
                Query<'_, (), Or<(Changed<A>, Changed<B>)>>,
                Query<'_, (), Or<(Added<A>, Added<B>)>>,
            )>,
        ) {
            let changed = set.q0().iter().count();
            let added = set.q1().iter().count();

            assert_eq!(changed, 1);
            assert_eq!(added, 1);

            *ran = true;
        }

        let mut world = World::default();
        world.insert_resource(false);
        world.spawn().insert_bundle((A, B));

        run_system(&mut world, query_system);

        assert!(*world.get_resource::<bool>().unwrap(), "system ran");
    }

    #[test]
    fn changed_resource_system() {
        struct Added(usize);
        struct Changed(usize);
        fn incr_e_on_flip(
            value: Res<'_, bool>,
            mut changed: ResMut<'_, Changed>,
            mut added: ResMut<'_, Added>,
        ) {
            if value.is_added() {
                added.0 += 1;
            }

            if value.is_changed() {
                changed.0 += 1;
            }
        }

        let mut world = World::default();
        world.insert_resource(false);
        world.insert_resource(Added(0));
        world.insert_resource(Changed(0));

        let mut schedule = Schedule::default();
        let mut update = SystemStage::parallel();
        update.add_system(incr_e_on_flip);
        schedule.add_stage("update", update);
        schedule.add_stage(
            "clear_trackers",
            SystemStage::single(World::clear_trackers.exclusive_system()),
        );

        schedule.run(&mut world);
        assert_eq!(world.get_resource::<Added>().unwrap().0, 1);
        assert_eq!(world.get_resource::<Changed>().unwrap().0, 1);

        schedule.run(&mut world);
        assert_eq!(world.get_resource::<Added>().unwrap().0, 1);
        assert_eq!(world.get_resource::<Changed>().unwrap().0, 1);

        *world.get_resource_mut::<bool>().unwrap() = true;
        schedule.run(&mut world);
        assert_eq!(world.get_resource::<Added>().unwrap().0, 1);
        assert_eq!(world.get_resource::<Changed>().unwrap().0, 2);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_mut_system() {
        fn sys(_q1: Query<'_, &mut A>, _q2: Query<'_, &mut A>) {}

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    fn disjoint_query_mut_system() {
        fn sys(_q1: Query<'_, &mut A, With<B>>, _q2: Query<'_, &mut A, Without<B>>) {}

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    fn disjoint_query_mut_read_component_system() {
        fn sys(_q1: Query<'_, (&mut A, &B)>, _q2: Query<'_, &mut A, Without<B>>) {}

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_immut_system() {
        fn sys(_q1: Query<'_, &A>, _q2: Query<'_, &mut A>) {}

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    fn query_set_system() {
        fn sys(mut _set: QuerySet<(Query<'_, &mut A>, Query<'_, &A>)>) {}
        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_with_query_set_system() {
        fn sys(_query: Query<'_, &mut A>, _set: QuerySet<(Query<'_, &mut A>, Query<'_, &B>)>) {}

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_sets_system() {
        fn sys(
            _set_1: QuerySet<(Query<'_, &mut A>,)>,
            _set_2: QuerySet<(Query<'_, &mut A>, Query<'_, &B>)>,
        ) {
        }

        let mut world = World::default();
        run_system(&mut world, sys);
    }

    #[derive(Default)]
    struct BufferRes {
        _buffer: Vec<u8>,
    }

    fn test_for_conflicting_resources<Param, S: IntoSystem<(), (), Param>>(sys: S) {
        let mut world = World::default();
        world.insert_resource(BufferRes::default());
        world.insert_resource(A);
        world.insert_resource(B);
        run_system(&mut world, sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_system_resources() {
        fn sys(_: ResMut<'_, BufferRes>, _: Res<'_, BufferRes>) {}
        test_for_conflicting_resources(sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_system_resources_reverse_order() {
        fn sys(_: Res<'_, BufferRes>, _: ResMut<'_, BufferRes>) {}
        test_for_conflicting_resources(sys);
    }

    #[test]
    #[should_panic]
    fn conflicting_system_resources_multiple_mutable() {
        fn sys(_: ResMut<'_, BufferRes>, _: ResMut<'_, BufferRes>) {}
        test_for_conflicting_resources(sys);
    }

    #[test]
    fn nonconflicting_system_resources() {
        fn sys(
            _: Local<'_, BufferRes>,
            _: ResMut<'_, BufferRes>,
            _: Local<'_, A>,
            _: ResMut<'_, A>,
        ) {
        }
        test_for_conflicting_resources(sys);
    }

    #[test]
    fn local_system() {
        let mut world = World::default();
        world.insert_resource(1u32);
        world.insert_resource(false);
        struct Foo {
            value: u32,
        }

        impl FromWorld for Foo {
            fn from_world(world: &mut World) -> Self {
                Self {
                    value: *world.get_resource::<u32>().unwrap() + 1,
                }
            }
        }

        fn sys(local: Local<'_, Foo>, mut modified: ResMut<'_, bool>) {
            assert_eq!(local.value, 2);
            *modified = true;
        }

        run_system(&mut world, sys);

        // ensure the system actually ran
        assert!(*world.get_resource::<bool>().unwrap());
    }

    #[test]
    fn remove_tracking() {
        let mut world = World::new();
        struct Despawned(Entity);
        let a = world.spawn().insert_bundle(("abc", 123)).id();
        world.spawn().insert_bundle(("abc", 123));
        world.insert_resource(false);
        world.insert_resource(Despawned(a));

        world.entity_mut(a).despawn();

        fn validate_removed(
            removed_i32: RemovedComponents<'_, i32>,
            despawned: Res<'_, Despawned>,
            mut ran: ResMut<'_, bool>,
        ) {
            assert_eq!(
                removed_i32.iter().collect::<Vec<_>>(),
                &[despawned.0],
                "despawning results in 'removed component' state"
            );

            *ran = true;
        }

        run_system(&mut world, validate_removed);
        assert!(*world.get_resource::<bool>().unwrap(), "system ran");
    }

    #[test]
    fn configure_system_local() {
        let mut world = World::default();
        world.insert_resource(false);
        fn sys(local: Local<'_, usize>, mut modified: ResMut<'_, bool>) {
            assert_eq!(*local, 42);
            *modified = true;
        }

        run_system(&mut world, sys.config(|config| config.0 = Some(42)));

        // ensure the system actually ran
        assert!(*world.get_resource::<bool>().unwrap());
    }

    #[test]
    fn world_collections_system() {
        let mut world = World::default();
        world.insert_resource(false);
        world.spawn().insert_bundle((42, true));
        fn sys(
            archetypes: &Archetypes,
            components: &Components,
            entities: &Entities,
            bundles: &Bundles,
            query: Query<'_, Entity, With<i32>>,
            mut modified: ResMut<'_, bool>,
        ) {
            assert_eq!(query.iter().count(), 1, "entity exists");
            for entity in query.iter() {
                let location = entities.get(entity).unwrap();
                let archetype = archetypes.get(location.archetype_id).unwrap();
                let archetype_components = archetype.components().collect::<Vec<_>>();
                let bundle_id = bundles
                    .get_id(std::any::TypeId::of::<(i32, bool)>())
                    .expect("Bundle used to spawn entity should exist");
                let bundle_info = bundles.get(bundle_id).unwrap();
                let mut bundle_components = bundle_info.components().to_vec();
                bundle_components.sort();
                for component_id in bundle_components.iter() {
                    assert!(
                        components.get_info(*component_id).is_some(),
                        "every bundle component exists in Components"
                    );
                }
                assert_eq!(
                    bundle_components, archetype_components,
                    "entity's bundle components exactly match entity's archetype components"
                );
            }
            *modified = true;
        }

        run_system(&mut world, sys);

        // ensure the system actually ran
        assert!(*world.get_resource::<bool>().unwrap());
    }

    #[test]
    fn get_system_conflicts() {
        fn sys_x(_: Res<'_, A>, _: Res<'_, B>, _: Query<'_, (&C, &D)>) {}

        fn sys_y(_: Res<'_, A>, _: ResMut<'_, B>, _: Query<'_, (&C, &mut D)>) {}

        let mut world = World::default();
        let mut x = sys_x.system();
        let mut y = sys_y.system();
        x.initialize(&mut world);
        y.initialize(&mut world);

        let conflicts = x.component_access().get_conflicts(y.component_access());
        let b_id = world
            .components()
            .get_resource_id(TypeId::of::<B>())
            .unwrap();
        let d_id = world.components().get_id(TypeId::of::<D>()).unwrap();
        assert_eq!(conflicts, vec![b_id, d_id]);
    }

    #[test]
    fn query_is_empty() {
        fn without_filter(not_empty: Query<'_, &A>, empty: Query<'_, &B>) {
            assert!(!not_empty.is_empty());
            assert!(empty.is_empty());
        }

        fn with_filter(not_empty: Query<'_, &A, With<C>>, empty: Query<'_, &A, With<D>>) {
            assert!(!not_empty.is_empty());
            assert!(empty.is_empty());
        }

        let mut world = World::default();
        world.spawn().insert(A).insert(C);

        let mut without_filter = without_filter.system();
        without_filter.initialize(&mut world);
        without_filter.run((), &mut world);

        let mut with_filter = with_filter.system();
        with_filter.initialize(&mut world);
        with_filter.run((), &mut world);
    }

    #[test]
    #[allow(clippy::too_many_arguments)]
    fn can_have_16_parameters() {
        fn sys_x(
            _: Res<'_, A>,
            _: Res<'_, B>,
            _: Res<'_, C>,
            _: Res<'_, D>,
            _: Res<'_, E>,
            _: Res<'_, F>,
            _: Query<'_, &A>,
            _: Query<'_, &B>,
            _: Query<'_, &C>,
            _: Query<'_, &D>,
            _: Query<'_, &E>,
            _: Query<'_, &F>,
            _: Query<'_, (&A, &B)>,
            _: Query<'_, (&C, &D)>,
            _: Query<'_, (&E, &F)>,
        ) {
        }
        fn sys_y(
            _: (
                Res<'_, A>,
                Res<'_, B>,
                Res<'_, C>,
                Res<'_, D>,
                Res<'_, E>,
                Res<'_, F>,
                Query<'_, &A>,
                Query<'_, &B>,
                Query<'_, &C>,
                Query<'_, &D>,
                Query<'_, &E>,
                Query<'_, &F>,
                Query<'_, (&A, &B)>,
                Query<'_, (&C, &D)>,
                Query<'_, (&E, &F)>,
            ),
        ) {
        }
        let mut world = World::default();
        let mut x = sys_x.system();
        let mut y = sys_y.system();
        x.initialize(&mut world);
        y.initialize(&mut world);
    }

    #[test]
    fn read_system_state() {
        #[derive(Eq, PartialEq, Debug)]
        struct A(usize);

        #[derive(Eq, PartialEq, Debug)]
        struct B(usize);

        let mut world = World::default();
        world.insert_resource(A(42));
        world.spawn().insert(B(7));

        let mut system_state: SystemState<(
            Res<'_, A>,
            Query<'_, &B>,
            QuerySet<(Query<'_, &C>, Query<'_, &D>)>,
        )> = SystemState::new(&mut world);
        let (a, query, _) = system_state.get(&world);
        assert_eq!(*a, A(42), "returned resource matches initial value");
        assert_eq!(
            *query.single().unwrap(),
            B(7),
            "returned component matches initial value"
        );
    }

    #[test]
    fn write_system_state() {
        #[derive(Eq, PartialEq, Debug)]
        struct A(usize);

        #[derive(Eq, PartialEq, Debug)]
        struct B(usize);

        let mut world = World::default();
        world.insert_resource(A(42));
        world.spawn().insert(B(7));

        let mut system_state: SystemState<(ResMut<'_, A>, Query<'_, &mut B>)> =
            SystemState::new(&mut world);

        // The following line shouldn't compile because the parameters used are not ReadOnlySystemParam
        // let (a, query) = system_state.get(&world);

        let (a, mut query) = system_state.get_mut(&mut world);
        assert_eq!(*a, A(42), "returned resource matches initial value");
        assert_eq!(
            *query.single_mut().unwrap(),
            B(7),
            "returned component matches initial value"
        );
    }

    #[test]
    fn system_state_change_detection() {
        #[derive(Eq, PartialEq, Debug)]
        struct A(usize);

        let mut world = World::default();
        let entity = world.spawn().insert(A(1)).id();

        let mut system_state: SystemState<Query<'_, &A, Changed<A>>> = SystemState::new(&mut world);
        {
            let query = system_state.get(&world);
            assert_eq!(*query.single().unwrap(), A(1));
        }

        {
            let query = system_state.get(&world);
            assert!(query.single().is_err());
        }

        world.entity_mut(entity).get_mut::<A>().unwrap().0 = 2;
        {
            let query = system_state.get(&world);
            assert_eq!(*query.single().unwrap(), A(2));
        }
    }

    #[test]
    #[should_panic]
    fn system_state_invalid_world() {
        let mut world = World::default();
        let mut system_state = SystemState::<Query<'_, &A>>::new(&mut world);
        let mismatched_world = World::default();
        system_state.get(&mismatched_world);
    }

    #[test]
    fn system_state_archetype_update() {
        #[derive(Eq, PartialEq, Debug)]
        struct A(usize);

        #[derive(Eq, PartialEq, Debug)]
        struct B(usize);

        let mut world = World::default();
        world.spawn().insert(A(1));

        let mut system_state = SystemState::<Query<'_, &A>>::new(&mut world);
        {
            let query = system_state.get(&world);
            assert_eq!(
                query.iter().collect::<Vec<_>>(),
                vec![&A(1)],
                "exactly one component returned"
            );
        }

        world.spawn().insert_bundle((A(2), B(2)));
        {
            let query = system_state.get(&world);
            assert_eq!(
                query.iter().collect::<Vec<_>>(),
                vec![&A(1), &A(2)],
                "components from both archetypes returned"
            );
        }
    }
}
