# Entity Component System - Design

The Entity Component System (ECS) is used by the engine to facilitate data-oriented processing. It allows the association of a number of components (data) to each entity (a fixed tag/id), and also the registration of a number of systems (processing) that will operate on each entity.

## Concepts

We want component definitions to be declarative. There should be no need to implement any functions when adding a new component type to the engine.

Systems on the other hand should not require any state. When processing, for each entity that matches with their signature, they have access to all the components that they require.

Systems will be scheduled so as to take into account their access requirements. All mutable accesses (writes) to a given component type should precede immutable accesses (reads). The system scheduler should be able to detect any invalid dependencies so that the graph remains acyclic.

There are two flavors of the engine: one that can operate with offline data (for edition), and one that uses runtime (compiled) data. The ECS will be used with the version of the engine that uses runtime data, as there is no real need for data processing when editing. The underlying architecture should be geared towards providing fast iteration and querying; adding and removing components to an entity after it has been instantiated should seldomly occur.

Entities themselves are owned by worlds. An expansive world can be subdivided into sub-worlds (loading cells).

Systems and worlds are owned by projects. A project represents a high-level concept such as a game title.

## Research / open questions

* What is the best way to express "global" state? Have components that are not associated with an entity? Alternatively, use world/project components? 
* What is the best way to express relationship between entities, like hierarchies for example. Could a shared component work?
  > A good example of a relationship, from R6 Siege, is when the player takes a hostage and constrains his movement/actions
* How to implement tags/markers on entities? Use empty components?
* Scripting
  * How should we allow users to define new component types? What would be the data-definition language?
  * How should we allow users to define new systems? What would be the scripting language?
* Hot reloading... how can we allow the dynamic update of entities?
* How are entities and their components instantiated during the serialization process?
* For immutable assets, such as Textures, Materials, etc... the components should simply refer to them (stored in resource manager)
* Do we need the notion of events for communication between systems?
* Should we support temporary components, like a connection for example?

## Tasks

- [x] Need to be able to analyze a system signature to extract component dependencies (both read and write)
- [ ] Components should allow custom attributes that can apply for the component as a whole, or on individual attributes. An example could be a "Name" component that is useful in edition mode and also in runtime, *except* if running a release (highly optimized) build.
- [ ] Try to implement an "attachment" scenario, like a weapon attached to player (entity relationship)
