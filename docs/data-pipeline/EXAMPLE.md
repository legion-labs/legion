# Examples:

Below you can find examples of how data pipeline works to transform data from a psd/fbx format to the runtime binary one.

The process in high level can be described as:

SOURCE => OFFLINE => RUNTIME ( OBJECT, PACKED ) => ARCHIVED

## Actor Example

This example illustrates data pipeline for a 'hero' actor which consists of 2 meshes, with 2 different materials using different textures, a skeleton and 2 animations accompanied by an animation blend tree.

The purpuse of this example is to illustrate various forms the data takes and the transitions that happen between them.

#### Source Representation

Optimized for artist's workflow; focused around DCC tools.
```
+ data-source/
  - hero.anim.fbx             // animation editing workflow
  - hero.anim.fbx.export      // metadata used to export into offline format
  - hero.materials.psd        // material texture editing workflow
  - hero.materials.psd.export // metadata used to export into offline format
  - hero.model.blend          // model & rigging creation workflow
  - hero.model.blend.export   // metadata used to export into offline format
  // animations blend trees created by editing offline format directly
```

#### Offline Representation

Optimized for tool editing (writing) & asset compilation; stripped from excess DCC format data.

Filenames are in reality GUIDs (replaced below by names for simplicity). File names are stored in .meta for display.
```
+ data-offline/
  - hero.actor              // top level
  - hero.actor.meta         // geom, skeleton, anim bundle(s) refs
  - hero.geom               // vertex data of many meshes
  - body.geom.meta          // material(s) refs
  - male.skeleton           // nodetree
  - male.skeleton.meta      // guid only, no refs
  - body.material           // texture refs, material properties
  - body.material.meta      // texture refs
  - hair.material           // texture refs, material properties
  - hair.material.meta      // texture refs
  - albedo.texture          // pixel data
  - albedo.texture.meta     // guid only, no refs
  - albedo2.texture         // pixel data
  - albedo2.texture.meta    // guid only, no refs
  - normal.texture          // pixel data
  - normal.texture.meta     // guid only, no refs
  - hero.anim.bundle        // sampling/compression, grouping
  - hero.anim.bundle.meta   // anim(s), blend tree references
  - idle.anim               // splines, keyframes
  - idle.anim.meta          // skeleton ref (or in bundle?)
  - run.anim                // splines, keyframes
  - run.anim.meta           // skeleton ref (or in bundle?)
  - hero.animbt             // for bt editor
  - hero.animbt.meta        // anim(s), skeleton references
```
- `offline resource` is a unit of data appearing in the offline resource browser
- each offline resource is accompanied with a limited in size `metadata` file
  - metadata file is a type-specific bag of properties opaquely parsable
  - it contains forward references (resources it depends on)
- backward references (resources that depend on it) need to be looked up
- data compilers read metadata files and can optionally open the resource itself

#### Object Representation

Optimized for engine runtime (reading); stripped from editor-related data.

Each 'data object' can be loaded in a different place in memory and keeps the system coupling to minimum.
```
+ data-obj/
  - albedo.texture  // platform-compressed
  - albedo2.texture // platform-compressed
  - normal.texture  // platform-compressed
  - male.skeleton   // skinning matrices, nodetree
  - idle.anim       // references skeleton
  - run.anim        // references skeleton
  - hero.animbt     // references animations
  - body.mesh       // references material
  - hair.mesh       // references material
  - body.material   // references textures
  - hair.material   // references textures
```

#### Packed Representation

Optimized for loading and patching; combines many related `data objects` into one file. Not all data objects need to be packed.
```
+ data-runtime/
  - albedo.texture
  - albedo2.texture
  - normal.texture
  - hero.animbundle     // bag of animations & blend trees (internal + external refs)
  - hero.materialbundle // bag of materials (external refs)
  - hero.model          // meshes & skeleton (internal refs)
```

Visual representation
```
  ======fileA======   =====fileC=====        =====fileD======
  |  body.mesh<--\|<->|body.material|<---+-->|albedo.texture|
  |  hair.mesh<--+|<->|hair.material|<\  :   ================
/>|male.skeleton</|   =============== :  :   =====fileE======
: =================                   :  \-->|normal.texture|
:                                     :      ================
: ======fileB=====                    :      ======fileF======
+>|   run.anim   |                    \----->|albedo2.texture|
\>|  idle.anim   |                           =================
  |  hero.animbt |
  ================
```

#### Archived Representation

Optimized for IO throughput; 'zips' many packed and object representations into one file.

```
characters.archive
intro_map.archive
common.archive
...
```
## Data Model Example

todo

## Game World/Level

todo
