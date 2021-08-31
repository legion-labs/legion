# Project Resources

The user-facing representation of this data pipeline component is an editor's 'Resource Browser'.

This component is responsible for adding/removing/moving resources in a form of source or offline data.

It provides a view of the resources present under source control and keeps track of local resource modifications (including adds, removes) before those are committed. To do all this it closely interacts with the source control system.

Example below illustrates file structure related to **source** and **offline** resources:

```ignore
+ project/
| + source/
| | - textures.psd        // source asset
| | - textures.export     // parameters of .psd export process
| + offline/
| | - albedo.texture      // output of the export process
| | - albedo.texture.meta // meta information of the output asset (name, dependencies, checksum)
| | - normal.texture
| | - normal.texture.meta 
| - project.index         // index of .meta & .export files
```
###### NOTE: GUIDs would be used for offline file names; here changed to 'name' to illustrate the example

##### Source Files with Export Properties

Source files usually are in form of DCC files stored in source-control with export metadata inside `.export` file. Those are exported by the user into `offline format` that must be kept in sync with its source files.

> DCC files are not exclusively treated as `source files`. It is possible to have them as `source files` and be part of the data compilation process.

##### Project Index

It provides an index of *offline resources* that can be inspected in the tool's 'Resource View'. Each resource is identifiable by a GUID while its name/path is stored as a property in an associated `.meta` file.

> All file names use resources's GUID instead of a human-readable name. This is to allow for easy renaming and moving of resources.

##### Build Dependency Tracking

Project resource management tracks build dependencies and stores them in a `.meta` file for each resource. This way the asset build system does not need to open the content of the asset to extract dependencies.