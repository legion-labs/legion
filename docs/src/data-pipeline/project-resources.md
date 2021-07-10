# Project Resources

The user-facing representation of this data pipeline component is an editor's 'Resource Browser'.

This component is responsible for adding/removing/moving resources in a form of source or offline data.

It provides a view of the resources present under source control and keeps track of local resource modifications (including adds, removes) before those are committed. To do all this it closely interacts with the source control system.

An example of file structure related to the component looks like this (GUIDs would be used for offline file names; here changed to 'name' to illustrate the example):

```ignore
+ data/
  - project.index       // index of .meta & .export files
  - textures.psd		// source asset
  - textures.export		// parameters of .psd export process
  - albedo.texture		// output of the export process
  - albedo.texture.meta	// meta information of the output asset (name, dependencies, md5)
  - normal.texture
  - normal.texture.meta 
```

##### Sources with Export Properties

DCC source files are stored in source-control with export metadata inside `.export` file. Those are exported by the user into `offline format` that must be kept in source-control in sync with source files.

##### Index of Offline Resources

It provides an index of offline resources that can be inspected in the tool's 'Resource View'. Each resource is identifiable by a GUID.

##### Build Dependency Tracking

It tracks build dependencies and stores them in a `.meta` file for each resource. This way the asset build system does not need to open the content of the asset to extract dependencies.