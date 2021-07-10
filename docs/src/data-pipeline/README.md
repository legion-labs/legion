# Data Pipeline

> **Disclaimer**: This document is work in progress and describes our aspirations. There are technical challenges to sort out so the final outcome might look different from the original vision described here.

This section provides information about data processing pipeline - everything from exporting from source files to loading a runtime asset.

#### DCC Tools Export

We use Digital Content Creation tools such as Maya, Blender, Photoshop to edit assets like textures, animations, etc. Some other assets are edited in our proprietary tools, like scene or level description.

#### Offline and Runtime Data Formats

Our tools and the engine use different data formats with the purpose of adjusting them to their respective needs:
- *offline format* - used by the tools; is optimized for writing.
- *runtime format* - used by the engine; is optimized for reading. 

One of the goals of the runtime format is to do as little post-load processing of the data as possible to achieve fast load times.

##### Data Model

[`Data Model`] chapter is dedicated to the problem of declaring offline and runtime data formats, data edition, compilation and many other related matters.

#### Focus on Many Different Workflows

Our intention is to be able to do what is best for each workflow individually. We believe fast iteration time is key in enabling creativity. Different types of content creation requires different solutions - some use dedicated tools that operate on data in offline format others require an in-engine preview.

#### Enable Small and Big Teams to Collaborate Efficiently

In order to support big teams working collaboratively it is our goal to minimize source control contention.

#### Patching & Live-Editing

Support for live games is at our core. We intend to provide the ability to control the way game's content is distributed with a focus on efficient changing of that content over the product's lifetime.

Patching will enable live editing of runtime assets by hot-reloading data. This will allow us to bring fast iteration to certain workflows.