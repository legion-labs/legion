# Data Pipeline - Context

This document describes assumptions related to the data pipeline. It provides context for which the data pipeline is developed. You can refer to [EXAMPLES DOCUMENT](./data-pipeline/EXAMPLE.md) to find more concrete use cases that the data pipeline needs to support.

#### Engine / Tool Context

- We use DCC tools (Maya, Blender, Photoshop, etc.) to edit assets like textures, animations. Some other assets are edited in our own editor(s) like levels.
- Engine runtime uses a different data format than the tools.
- Some workflows will rely solely on offline tools; other workflows will take advantage of "live editing" - hot reloading of runtime data.
- We focus on enabling small and big teams to collaborate efficiently. The way tool data is structured must support seamless collaboration. 
- The loaded runtime asset should require the least amount of CPU work to be ready to be used.

#### Data Model

- We use a data definition language to express all offline data formats and some runtime data formats for some kinds of assets (configuration files, entity prefabs, scene description, etc.).
- It supports nested objects, parenting/overriding properties, property grid edition, runtime (in offline format) & build-time (in runtime format) transformations, versioning & version conversion

> How do we generate code? Do we want a more generic code generation platform that would support other needs?

#### Patching / Live Editing

- As we intend to support live games patching is a first-class feature. The ability to create small patches is key.
- Patching will be used to enable live editing of runtime assets by hot reloading data where it matters for quick iteration.
