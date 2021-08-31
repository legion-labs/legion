# Data Build

Data build's primary responsibility is to transform resources from their offline format into the runtime format. In order to maintain fast iteration time at all scales it is key to keep its processing time to minimum. It does it by implementing incremental building, efficient dependency tracking, data build output caching.

The output of an asset build process is a `manifest file` containing the list of `compiled resources` tuples of *(GUID, size, checksum)*. The compiled resource can be either in a runtime format consumed by the engine, intermediate format that undergoes further processing or a format that is less runtime-optimized and is used by the editor. All compiled resources are stored in the `compiled content store` - a content-addressable data storage. 

Offline resources describe build dependencies in `.meta` files which are parsed by the asset build process in order to extract the dependency structure without loading the content of the offline resource itself.

### DCC Exporters

DCC exporters are command-line tools that are used to extract offline resources from DCC files into a structured format parsable by editing tools. Those are usually exectued on demand, when a change to the source resource is done by the artist.

It is also possible to have DCC exporter run as part of the data build process. For this, a source DCC file must be versioned under **offline/** directory (in contrary to **source/** directory)

## `AssetPathId` and Data Compilers

Data build defines a framework for data processing. It's main components are: **data compilers**, **content types** and **AssetPathId**.

At its core it operates on **source resources** - files that contain user-defined data. Each source resource has a **Content Type** assigned to it.

A simple **AssetPathId** can define a transformation from a concrete **source resource** to a **derived resource**. This transformation is represented by a tuple (Content Type, Content Type) which is supported by a **Data Compiler**.

A more complex **AssetPathId** can represent a series of transformations, always starting from a **source resource**, that creates a series of **derived resource** - where next transformation's input is the previous transformation's output.

**Data Compiler** can read the **input resource** (either **source resource** or a **derived resource**), it's depenencies and can output one or more new resources.

Each compilation unit has to have all its inputs declared ahead of time. The data processed by the compiler needs to be strictly limited to those inputs making the build process **hermetic**.

## Compiler Code & Resource Format Versioning

Data build process handles changes to compiler code and asset format changes by making sure relevant assets are recompiled.

## Dependency Processing

Data build process reads asset dependencies from `.meta` files and those defined by the `AssetPathId` in order to determine the order in which the assets need to be processed and to determine if an asset needs to be rebuilt.

A dependency filtering (by type) is required to be able to reduce the dependencies for various processes (i.e. include only the geometry in navmesh generation).

## Specialized Output (i.e. platform, game version)

Each compiler can specialize its output based on various build parameters like `target platform` or `game version/features`. This can be used to generate things like: platform-specific texture format, exclude certain game features, etc.

## Specialized Input (i.e. language, region)

Each compiler can take different input based on various build parameters like `target region` or `language`. This can be used for things like: region-specialized assets, languages, etc.

## Compiled Content Store

Each compiled resource is stored in a cache. This cache is used to download data on demand when a game process is run. It also serves a purpose of sharing compilation results between users to speed up build times.

## Build distribution

The build process must be distributable to take advantage of multiple machine's processing power. This excludes assets that are IO-bound.