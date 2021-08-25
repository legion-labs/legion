# Runtime Asset Management

This is the runtime part of the system responsible for loading resources at runtime and their memory management

It needs to support two different use cases - the runtime and offline resource management. For the tools and editor use cases.

##### Resource Lifetime / Loading Composite Resources

The runtime needs to ensure there is only one copy of each unique resource loaded in memory, manage cross-references between them and make sure the dependent resources are loaded when the parent resource loading is requested.

Assets are identified by compiled asset GUID instead of file paths.

Some assets will require post-load processing - but this should be kept to minimum.

##### Memory management

Memory management is a complex topic on its own. It needs to be kept in mind that different assets might be loaded into different types of memory; different allocation strategies might be used for streaming, etc.