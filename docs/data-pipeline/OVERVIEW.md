# Data Pipeline - Overview

> **Disclaimer**: This document is work in progress and describes our aspirations. There are technical challenges to sort out so the final outcome might look different from the original vision described here.

This document describes the overview of the data processing pipeline in the context outlined in the [CONTEXT DOCUMENT](./data-pipeline/CONTEXT.md).

In high-level the data pipeline deals with 4 distinct representations of data and 3 data transformation processes converting those.

>  **SOURCE** =export=> **OFFLINE** =compile=> **RUNTIME** =package=> **ARCHIVE**

Data representations:

- *Source* - data in source editor format (Photoshop, Maya, etc.)
- *Offline* - edition-friendly data format optimized for editing workflow and writing. In in-house opaquely parsable format (i.e. bag of properties)
- *Runtime* - runtime-friendly data format optimized for reading. In binary format.
- *Archive* - IO-friendly data format optimized for throughput. in zip-like archive format ('virtual FS')

Data transformations:

- *Export* - process of converting source format into offline format. Stripping from DCC related data.
- *Compilation* - process of converting offline format into runtime format. Conversion to runtime friendly representation and grouping of related data.
- *Packaging* - process of converting runtime format into an archived format. Includes compression.

## Cross-cutting Concerns

Here are the core principles around the data pipeline.

1. Data build determinism
2. Support for variety of platforms, languages, regions, versions, etc.
3. Sharing of compiled resources (with others; across branches & builds)
4. Everything is built incrementally / with minimal rebuilds
5. Runtime assets are fetched on demand; at runtime from the cache. (for clients, servers, tools).

## Components

This section describes high level detail about the major components of the data pipeline.

The data pipeline consists of 4 major components:

- offline resource management
- asset build process
- runtime resource management
- resource packaging

#### OFFLINE RESOURCE MANAGEMENT

The user-facing representation of this component is a 'Resource Browser' in the Editor.

This component is responsible for adding/removing/moving resources in a form of source or offline data.

It provides a view of the resources present under source control and keeps track of local resource modifications (including adds, removes) before those are committed. To do all this is closely interacts with the source control system.

An example of file structure related to the component looks like this (GUIDs would be used for offline file names; here changed to 'name' to illustrate the example):

```
+ data/
  - textures.psd		// source asset
  - textures.export		// parameters of .psd export process
  - albedo.texture		// output of the export process
  - albedo.texture.meta	// meta information of the output asset (name, dependencies, md5)
  - normal.texture
  - normal.texture.meta 
```

##### 	Sources with Export Properties

DCC source files are stored in source-control with export metadata inside `.export` file. Those are exported by the user into `offline format` that must be kept in source-control in sync with source files.

##### 	Index of Offline Resources

It provides an index of offline resources that can be inspected in the tool's 'Resource View'. Each resource is identifiable by a GUID.

##### 	Build Dependency Tracking

It tracks build dependencies and stores them in a `.meta` file for each resource. This way the asset build system does not need to open the content of the asset to extract dependencies.

#### ASSET BUILD PROCESS & COMPILED ASSET CACHE

Asset build's primary responsibility is to transform assets from the offline format into the runtime format. In order to maintain fast iteration time at all scales it is key to keep its processing time to minimum. It does it by implementing incremental building, efficient dependency invalidation, compilation output caching.

The output of an asset build process is a `manifest file` containing the list of `compiled resources` - GUIDs, sizes, md5s. A compiled resource is a file containing asset(s) in runtime consumable format. The resources itself are stored in the `compiled asset cache`. 

Offline resources describe build dependencies in `.meta` files which are parsed by the asset build process in order to extract the dependency structure without loading the content of the offline resource itself.

##### 	DCC Exporters

DCC exporters are command-line tools that are used to extract offline resources from DCC files into a structured format parsable by engine tools.

##### 	Asset Types and Compilers

Asset build process is able to identify and process various kinds of assets. The list of asset types and associated compilers is easily extendable. Compilers are standalone command-line tools that are used to convert from offline to runtime format.

Each compilation unit has to have all its inputs declared ahead of time. The data processed by the compiler needs to be strictly limited to those inputs.

##### 	Compiler Code & Asset Format Versioning

Asset build process handles changes to compiler code and asset format changes by making sure relevant assets are recompiled.

##### 	Dependency Processing (compilation order and invalidation)

Asset build process reads asset dependencies from `.meta` files in order to determine the order in which the assets need to be processed and to determine if an asset needs to be rebuilt.

A dependency filtering (by type) is required to be able to reduce the dependencies for various processes (i.e. include only the geometry in navmesh generation).

##### 	Specialized Output (i.e. platform, game version)

Each compiler can specialize its output based on various build parameters like `target platform` or `game version/features`. This can be used to generate things like: platform-specific texture format, exclude certain game features, etc.

##### 	Specialized Input (i.e. language, region)

Each compiler can take different input based on various build parameters like `target region` or `language`. This can be used for things like: region-specialized assets, languages, etc.

##### 	Compiled Asset Cache

Each compiled asset is stored in a cache. This cache is used to download data on demand when a game process is run. It also serves a purpose of sharing compilation results between users to speed up build times.

##### 	Build distribution

The build process must be distributable to take advantage of multiple machine's processing power. This excludes assets that are IO-bound.

#### RUNTIME RESOURCE MANAGEMENT

This is the runtime part of the system responsible for loading resources at runtime and their memory management

It needs to support two different use cases - the runtime and offline resource management. For the tools and runtime use cases.

##### 	Resource Lifetime / Loading Composite Resources

The runtime needs to ensure there is only one copy of each unique resource loaded in memory, manage cross-references between them and make sure the dependent resources are loaded when the parent resource loading is requested.

Assets are identified by compiled asset GUID instead of file paths.

Some assets will require post-load processing - but this should be kept to minimum.

##### 	Memory management

Memory management is a complex topic on its own. It needs to be kept in mind that different assets might be loaded into different types of memory; different allocation strategies might be used for streaming, etc.

#### RESOURCE PACKAGING

This part of the data pipeline is responsible for taking the output of the `asset build` - the manifest file - downloading the indicated resources from `compiled asset store` and creating archive files containing listed files.

During the development it is responsible for executing this process in a 'just-in-time' manner

##### 	"Virtual Filesystem" Archive

To reduce file operations and seek overhead the game uses `.archive` files in a zip-like format (depending on the platform). It needs to support .archive prioritization and overriding of runtime resources (files) to support data patching.

##### 	Expandable Archive

When running a development build the assets are never present at boot. A game executable and a `.manifest` file is enough to run. 

The `.manifest` file can contain a list of addresses to `compiled asset cache` instances. Upon load request we first check if the resource is present in a local `.archive`. If it is not there it is downloaded from one of the remote cache instances and stored in a local expandable `.archive`. On the next run the resource will be already present.

This local expandable archive can persist between different game versions - making it very efficient to download build's content as only the files that change will have to be pulled from a remote source.

------

## References:
- [The Story behind The Truth: Designing a Data Model · Our Machinery](https://ourmachinery.com/post/the-story-behind-the-truth-designing-a-data-model/)
- [Data Schemas on Frostbite](https://www.ea.com/frostbite/news/a-tale-of-three-data-schemas)
- [HandmadeCon 2016 - Asset Systems and Scalability - YouTube](https://www.youtube.com/watch?v=7KXVox0-7lU)
- [HandmadeCon 2016 - Large-scale Systems Architecture - YouTube](https://www.youtube.com/watch?v=gpINOFQ32o0)
- [Scripting in Uncharted 2](https://www.slideshare.net/naughty_dog/statebased-scripting-in-uncharted-2-among-thieves)
- [Data Building Pipeline of Overwatch](https://www.gdcvault.com/play/1024019/The-Data-Building-Pipeline-of)
- Unreal Engine Runtime Resource Mgmnt: [Asset Mgmnt](https://docs.unrealengine.com/4.26/en-US/ProductionPipelines/AssetManagement/), [Cooking & Chunking](https://docs.unrealengine.com/4.26/en-US/SharingAndReleasing/Patching/GeneralPatching/CookingAndChunking/)

