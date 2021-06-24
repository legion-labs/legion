# Data Pipeline

This document describes problems, ideas, considerations related to the data processing pipeline. This refers to anything related to storing source assets, processing them and using data in the engine. 

In high-level the data pipeline deals with 3 types of data and 2 transformations used to convert between those types.

> **SOURCE** =<u>export</u>=> **OFFLINE** =<u>compilation</u>=> **RUNTIME** =<u>packing</u>=> **PACKAGED**

<u>Data formats</u>:

- *Source* - data in a source editor format (photoshop, maya, hudini, etc)
- *Offline* - edition-friendly data format optimized for editing/writing. in-house opaquely parsable format (i.e. bag of properties)
- *Runtime* - runtime-friendly data format optimized for reading. in binary format.
- *Packaged* - IO-friendly data format optimized for throughput. in zip-like archive format

<u>Transformations</u>:

- Data export - process of converting source format into offline format which is stripped of unrelevant pieces
- Data compilation - process of converting offline data format to runtime format
- Data packing - process of converting runtime format into packaged format

## Considerations:

Below is a list things to consider for each data type, transformation and some cross-cutting concerns.

### Source Data Format

1. Do we care about source data?

   - We could not care about the source data format and store only the offline data format. Which would imply on-demand conversion from offline to source data that we would have to maintain (i.e. opening source mesh file in Maya, Blender, etc)
   - What is the primary source of data? Source or offline? Or a mix?
   - Source assets are possible very large in size - we need to make sure they are not polluting IO if not necessary. (src ctl?)

### Offline Data Format

1. Data schema

   - What is the language that we define the schema in?
   - What is the language(s) that tools use?
   - Take a look at the Racket macro system as an example of an extensible data definition system

2. Asset metadata

   - Provide a way to describe additional data about an asset (unique id, compression, anim frame-range, etc)
   - Metadata can point to a source asset producing many offline assets from it
   - Have a central place for asset metadata (db?)

3. Versioning & Compatibility

   - Can a new tool version open old data?
   - Can an old tool version open new data?

4. Version Conversion

   - How can we run custom code to convert data from old to new format?
   - How can we read old format and write a new one at the same time?
   - How do we deprecate formats and remove conversion over time?

5. Dependency tracking (static & dynamic) consistent, asset type independent.

   - How do we determine dependencies without having to read asset's content?

6. Generic asset cloning

   - Can we copy/paste assets in an asset-type independent way?

7. Generic undo/redo

   - Can we create undo/redo functionality in an asset-independent way?

8. Seamless collaboration

   - What does it mean to collaborate seamlessly?
   - How do we resolve conflicts? Which conflicts can be auto-resolved?

9. Offline data granularity

   - To enable good collaboration data should be split in many files.

10. Incremental building

   - How do we track what changed?
   - How do we know what needs to be rebuilt?
   - How do we keep dependencies to minimum?

### Runtime Data Format

1. Data schema

   - How do we define offline data vs runtime data formats? Is it one definition? Or many?

2. Runtime dependencies

   - How are assets grouped so that they are loaded together?
   - Do we have statically vs dynamically loaded references?

3. Patching support

   - What's the granularity of data we patch? How is it structured to not break references?
   - How do we support live-edit? (without having to restart)

4. Streaming support

   - How do we support level-streaming?
   - How are the assets divided into streamed chunks?

5. Runtime reflection

   - Having the ability to target data properties by 'path' at runtime (i.e. scripting)
   - Being able to live-edit properties?

### Packaged Data Format

1. Support data compression
2. File "virtualization"

   - I.e. loading different `archive` for different language while keeping the same filepaths at runtime

3. Support patching

   - Creating `delta archives`
   - Overriding files through higher-priority `archives`

### Data Export (source -> offline)

1. This process can be either seen as an integral (at times automated) part of the data pipeline or artist's responsibility outside of the data pipeline.
2. Data pipeline can be aware of export tools, data pipeline & export metadata to automate data exporting.

### Data Compilation (offline -> runtime)

1. Transforming from one form to another

   - Stripping debug/editor data
   - Completely different representation 
   - Easily adding a new transformation process to data compilation

2. Dependencies

   - Being able to access the dependencies of the given transformation
   - Producing an output that can act as an input to another process

3. No editor should be involved in the compilation process

4. Have a way to output runtime load dependencies

5. Be able to run compilation step to enable live-editing (related to patching and data reloading?)

### Data Packing (runtime -> packaged)

1. Data compression: LZ, Kraken
2. Streaming-friendly data reorganization?
3. Batching for efficient loading?
4. Layered/priority loading to allow patches to override files

## Cross-cutting Concerns

- Unified scripting across all data types
- Data schema & code generation
  - Define data separately from logic
  - Define runtime and offline data format separately?
  - Support for domain-specific code generation

## Questions:

- Do we store source assets or the offline assets in source control?
- How do we display offline data in editor? is it compiled to runtime data? or we display offline data?
- Is it feasible/reasonable to have separate runtime and storage data schemas.





------



## References:
- [The Story behind The Truth: Designing a Data Model · Our Machinery](https://ourmachinery.com/post/the-story-behind-the-truth-designing-a-data-model/)
- https://www.ea.com/frostbite/news/a-tale-of-three-data-schemas
- [HandmadeCon 2016 - Asset Systems and Scalability - YouTube](https://www.youtube.com/watch?v=7KXVox0-7lU)
- [HandmadeCon 2016 - Large-scale Systems Architecture - YouTube](https://www.youtube.com/watch?v=gpINOFQ32o0)
- https://www.slideshare.net/naughty_dog/statebased-scripting-in-uncharted-2-among-thieves



