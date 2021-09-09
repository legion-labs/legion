# Sample Data

This folder contains manually crafted data. The idea is for this data to help us reason about data reflection, deprecation, data loading, data building, and editing. It uses RON to make for readability. This should evolve to being a large integration test data.

Current features :
* World modeling as a list of separate files we depend on, meaning that and add/remove of an entity will mean a resolve.
* Instancing of data, through a generic instance file, which depends on the original and contains overrides
* Inline dependency processing definition.
* Instancing a group and changing one of its dependencies.

What the data currently contains:
* A world (sample 1), a ground entity which is original to the world, instanced cubes
* The ground entity which depends on an explicit mesh (a non imported mesh),
* A Cube which depends on an exported geometry (from an obj file)
