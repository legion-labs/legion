# Entity Component Systems - Research

## Overviews / general links

- [Wikipedia entry](https://en.wikipedia.org/wiki/Entity_component_system)
- [ECS Subreddit](https://www.reddit.com/r/EntityComponentSystem/)
- [Entities, components and systems](https://medium.com/ingeniouslysimple/entities-components-and-systems-89c31464240d), Mark Jordan
- [ECS FAQ](https://github.com/SanderMertens/ecs-faq), Sander Mertens (author of Flecs)
- [Overwatch Gameplay Architecture and Netcode](https://www.youtube.com/watch?v=W3aieHjyNvw), Timothy Ford (Blizzard), GDC 2017 talk
- [Understanding data-oriented design for entity component systems](https://www.youtube.com/watch?v=0_Byw9UMn9g), Unity at GDC 2019
- [Options for Entity interaction](https://www.youtube.com/watch?v=KuGRkC6wzMY), Luna Meier, Unite Copenhagen 2019

## Common definitions

*Entities* are simply tags, unique identifiers.

*Components* are purely data structures (POD types), that contain data related to a specific domain such as position, physics, appearance, etc. Each component instance is associated with (owned by) an entity.

*Systems* perform data-transformation on a set of components. They will iterate on every entity that possesses all the component types they depend on.

## Architectures

### Archetypes

Entities are grouped together according to the set of components they possess. Each archetype can be thought of as a table with entities as rows and components as columns.

> Fast to query and iterate

### Sparse sets

Each component has a sparse set of entities (equivalent of a hashset)

> Fast add/remove operations

### Bitsets

Components are stored in arrays, indexed by the entity. A bitset is used to determine which components an entity possesses.

### Reactive

Uses signals from entity mutations to match them with systems.
> Note: Unclear if related to archetypes?

See [Entitas](https://github.com/sschmid/Entitas-CSharp)

## Rust implementations

### Overviews of using Rust for ECS

- [Entity Component System implementations](https://arewegameyet.rs/ecosystem/ecs/), Are we game yet?
- [Using Rust for game development](https://www.youtube.com/watch?v=aKLntZcp27M), Catherine West, RustConf 2018 closing keynote
    - [detailed notes](https://kyren.github.io/2018/09/14/rustconf-talk.html)
    - depends on [slotmap](https://crates.io/crates/slotmap) crate for generational indexes, and [AnyMap](https://crates.io/crates/anymap) crate
    - [Rant: Entity systems and the Rust borrow checker ... or something](https://www.youtube.com/watch?v=4t1K66dMhWk), Jonathan Blow
- [Specs and Legion, two very different approaches to ECS](https://csherratt.github.io/blog/posts/specs-and-legion/), Cora Sherratt
    - proposal to move Amethyst from specs to legion: [[RFC Discussion] Legion ECS Evolution](https://github.com/amethyst/rfcs/issues/22)
- [Data Oriented GUI in Rust](https://www.youtube.com/watch?v=4YTfxresvS8), Raph Levien, Bay Area Rust Meetup

### List of Rust implementations

| Name | Repo | Architecture | Popularity (downloads) | Notes |
| ---- | ---- | ------------ | ---------------------: | ----- |
| [Bevy](https://bevyengine.org/) | [GitHub](https://github.com/bevyengine/bevy) | ? | 54k | [Bevy ECS overview](https://bevyengine.org/learn/book/getting-started/ecs/). Entities are simple type containing unique integer. Components are  normal Rust structs. Additional [research notes](https://hackmd.io/0jXdWYR3QQ6uycnbD3Oc_g) on HackMD. |
| [DCES](https://crates.io/crates/dces) | [Gitlab](https://gitlab.redox-os.org/redox-os/dces-rust) | ? | 11k | part of OrbTk |
| [Dotrix](https://crates.io/crates/dotrix) | [GitHub](https://github.com/lowenware/dotrix) | ? | <1k | |
| ecs-rs | [GitHub](https://github.com/HeroesGrave/ecs-rs) | ? | ? | port of [Artemis framework](https://gamadu.com/artemis/) |
| hecs | [GitHub](https://github.com/Ralith/hecs) | archetype | 12k | |
| [Legion](https://crates.io/crates/legion) | [GitHub](https://github.com/amethyst/legion) | archetype | 33k | Queries specify components using types (with mut for write-access). Systems use update functions, components extracted from signature, and scheduled according to data-flow |
| [Planck](https://jojolepro.com/blog/2021-01-13_planck_ecs/) | [GitHub](https://github.com/jojolepro/planck_ecs) | ? | <1k | Adds resources, which are independent of entites, ex: game-time |
| [Rustic](https://crates.io/crates/recs) | [GitHub](https://github.com/andybarron/rustic-ecs) | ? | 4k | |
| Shipyard | [GitHub](https://github.com/leudz/shipyard)  | sparse set | 4k | |
| [Specs](https://specs.amethyst.rs/docs) | [GitHub](https://github.com/amethyst/specs) | bitset | 229k | Part of [Amethyst](https://amethyst.rs/) engine. Intro in [The Specs book](https://specs.amethyst.rs/docs/tutorials/01_intro.html). Dependency on [hibitset](https://docs.rs/hibitset/0.6.3/hibitset/). Implement Component trait for each struct (ex: Position, Velocity); sub-type Storage implements the serialization. Implement System trait for system structs; sub-type SystemData defines access to different components, in a tuple; implement run method. World object acts as registry for components and systems |	

### Performance

- [ECS Bench Suite](https://github.com/rust-gamedev/ecs_bench_suite)

## C/C++/C# implementations

### Overviews of using C/C++/C# for ECS

- The Entity-Component-System - [An awesome game-design pattern in C++ (Part 1)](https://www.gamasutra.com/blogs/TobiasStein/20171122/310172/The_EntityComponentSystem__An_awesome_gamedesign_pattern_in_C_Part_1.php) and [BountyHunter game (Part 2)](https://www.gamasutra.com/blogs/TobiasStein/20171122/310174/The_EntityComponentSystem__BountyHunter_game_Part_2.php), Tobias Stein

### List of C/C++/C# implementations

| Name | Architecture | Notes |
| ---- | ------------ | ----- |
| [Entitas](https://github.com/sschmid/Entitas-CSharp) | reactive | Written in C#. Can integrate with Unity. [Entitas++](https://github.com/JuDelCo/Entitas-Cpp) is a C++ port. |
| [EntityX](https://github.com/alecthomas/entityx) | bitset | |
| [EnTT](https://github.com/skypjack/entt) | sparse set | EnTT is a header-only, tiny and easy to use library for game programming and much more written in modern C++. Among others, it's used in Minecraft by Mojang |
| [Flecs](https://github.com/SanderMertens/flecs) | archetype | |
| [The Machinery](https://ourmachinery.com/) | archetype | Written in plain C. Plugin-based. Graph-based import workflow. Centralized [data model](https://ourmachinery.com/post/the-story-behind-the-truth-designing-a-data-model/). Real-time collaborative game editing |
| [Unity](https://docs.unity3d.com/Packages/com.unity.entities@0.1/manual/index.html) | archetype | The Entity Component System (ECS) is the core of the Unity [Data-Oriented Technology Stack](https://unity.com/dots) (DOTS). Several [packages](https://unity.com/dots/packages) available in preview. |
| [Unreal](https://www.unrealengine.com/en-US/tech-blog/performance-at-scale-sequencer-in-unreal-engine-4-26) | archetype | In 4.26, the foundational runtime for Sequencer was redesigned to lay out evaluation data in a way that is cache-efficient and transparent through an evaluation framework based on the entity-component-system pattern. |
