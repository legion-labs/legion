# Packaging

This part of the data pipeline is responsible for taking the output of the `asset build` - the manifest file - downloading the indicated resources from `content store` and creating archive files containing listed files.

During the development it is responsible for executing this process in a 'just-in-time' manner

##### "Virtual Filesystem" Archive

To reduce file operations and seek overhead the game uses `.archive` files in a zip-like format (depending on the platform). It needs to support .archive prioritization and overriding of runtime resources (files) to support data patching.

##### Expandable Archive

When running a development build the assets are never present at boot. A game executable and a `.manifest` file is enough to run. 

The `.manifest` file can contain a list of addresses to `content store` instances. Upon load request we first check if the resource is present in a local `.archive`. If it is not there it is downloaded from one of the remote cache instances and stored in a local expandable `.archive`. On the next run the resource will be already present.

This local expandable archive can persist between different game versions - making it very efficient to download build's content as only the files that change will have to be pulled from a remote source.