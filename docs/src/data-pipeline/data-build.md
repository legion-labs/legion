# Data Build

Data build's primary responsibility is to transform resources from their offline format into the runtime format. In order to maintain fast iteration time at all scales it is key to keep its processing time to minimum. It does it by implementing incremental building, efficient dependency tracking, data build output caching.

The output of an asset build process is a `manifest file` containing the list of `compiled resources` - GUIDs, sizes, md5s. A compiled resource is an asset file in runtime consumable format. It is stored in the `compiled asset cache` - a content-addressable data storage. 

Offline resources describe build dependencies in `.meta` files which are parsed by the asset build process in order to extract the dependency structure without loading the content of the offline resource itself.

##### DCC Exporters

DCC exporters are command-line tools that are used to extract offline resources from DCC files into a structured format parsable by editing tools. For now, the data pipeline does not interact with them as exporting is considered as a step happening before data build.

##### Resource Types and Compilers

Data build process is able to identify and process various kinds of resources. The list of asset types and associated compilers is easily extendable. Compilers are standalone command-line tools that are used to convert from offline to runtime format.

Each compilation unit has to have all its inputs declared ahead of time. The data processed by the compiler needs to be strictly limited to those inputs making the build process **hermetic**.

##### Compiler Code & Resource Format Versioning

Data build process handles changes to compiler code and asset format changes by making sure relevant assets are recompiled.

##### Dependency Processing

Data build process reads asset dependencies from `.meta` files in order to determine the order in which the assets need to be processed and to determine if an asset needs to be rebuilt.

A dependency filtering (by type) is required to be able to reduce the dependencies for various processes (i.e. include only the geometry in navmesh generation).

##### Specialized Output (i.e. platform, game version)

Each compiler can specialize its output based on various build parameters like `target platform` or `game version/features`. This can be used to generate things like: platform-specific texture format, exclude certain game features, etc.

##### Specialized Input (i.e. language, region)

Each compiler can take different input based on various build parameters like `target region` or `language`. This can be used for things like: region-specialized assets, languages, etc.

##### Compiled Asset Cache

Each compiled asset is stored in a cache. This cache is used to download data on demand when a game process is run. It also serves a purpose of sharing compilation results between users to speed up build times.

##### Build distribution

The build process must be distributable to take advantage of multiple machine's processing power. This excludes assets that are IO-bound.