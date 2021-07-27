# Entity Component System - Design

The Entity Component System (ECS) is used by the engine to facilitate data-oriented processing. It allows the association of a number of components (data) to each entity (a fixed tag/id), and also the registration of a number of systems (processing) that will operate on each entity.

## Objectives

We want component definitions to be declarative. There should be no need to implement any functions when adding a new component type to the engine.

Systems on the other hand should not require any state. When processing, for each entity that matches with their signature, they have access to all the components that they require.

Systems will be scheduled so as to take into account their access requirements. All mutable accesses (writes) to a given component type should precede immutable accesses (reads).

The engine can operate with either offline (edition) or runtime (compiled) data. The ECS will be used with the version of the engine that uses runtime data, as there is no real need for data processing when editing. The underlying architecture should be geared towards provided fast iteration and querying; adding and removing components to an entity after it has been instantiated should occur seldomly.

Entities themselves are owned by worlds. An expansive world can be subdivided into sub-worlds (loading cells).

## Tasks

- Need to be able to analyze a system signature to extract component dependencies (both read and write)
- For "global" state, associate components without an entity. Alternatively, use world components?
- Components should allow custom attributes that can apply for the component as a whole, or on individual attributes. An example could be a "Name" component that is useful in edition mode and also in runtime, except if running an optimized/final build.
