# Design Overview

In high-level the data pipeline deals with 4 distinct representations of data and 3 data transformation processes between those formats.

>  **SOURCE** = export => **OFFLINE** = compile => **RUNTIME** = package => **ARCHIVE**

Data Representations:

- *Source* - data usually in DCC format; version-controlled; processed outside of data compilation.
- *Offline* - data in a format optimized for writing, debuggability and storage. In an in-house opaquely parsable format.
- *Runtime* - data in a format optimized for reading. In binary format.
- *Archive* - IO-firendly data format. In zip-like archive format ('virtual FS').

Data Transformations:

- *Export* - process of converting source format into offline format. Stripping from DCC related data.
- *Compilation* - process of converting offline format into runtime format. Conversion to runtime friendly representation and grouping of related data.
- *Packaging* - process of converting runtime format into an archived format. Includes compression.

## Cross-cutting Concerns

Here are the core principles that hold true across the data pipeline.

1. Data build determinism
2. Hermetic builds
3. Support for variety of platforms, languages, regions, versions, etc.
4. Always sharing of compilation results (across branches, versions, etc)
5. Everything is built incrementally / with minimal rebuilds
6. Runtime assets can be fetched on demand; at runtime from the cache.

## Components

The data pipeline consists of 4 major components:

- Project resource management
- Data build process
- Runtime asset management
- Asset packaging

The following sections describe them in detail.
