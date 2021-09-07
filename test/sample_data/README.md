# Sample Data

This folder contains manually crafted data. The idea is for this data to help us reason about data reflection, deprecation, data loading, data building, and editing. It uses RON to make for readability. This should evolve to being a large integration test data.

Current features :
* World modeling as a list of sepereate files we depend on, meaning that and add/remove of an entity will mean a resolve.
* Instancing of data, through a genering instance file, which depends on the original and contains overrides
* Inline dependency processing definition.
