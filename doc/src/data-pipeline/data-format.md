# DataContainer Format

Allow serialization and editing of data structures for runtime and editor process.
DataContainer are written as Rust struct, with custom attributes to extend functionnality.
Reflection is used for offline serialization

## Features 

* Data Format should allow Versioning of the struct to trigger data-pipeline rebuild. Use Reflection system to calculate struct hash signature. Fields addition, removal, rename, change in defaultValue/attributes should affect data hash signature.
* Support Default Value. Unchaged fields are skipped during offline serialization.
* Serialization should support fields deprecation
* DataContainer should support code inheritance to derive fields from DataContainer code definition (doable in Rust?)
* DataContainer should support data inheritance, allowing a DataContainer defaultValue to be defined from another DataContainer data, with support for overriding derived values
* Runtime representation should be compact and support in-place serialization (unsafe?)


## Property Types

* Should support most primitives types (String, Int, Float, etc)
* Should support Simple Enum
* Complex type (Color, Vec2, Vec3, Transform, Curve, other DataContainer)
* Array Types
* Reference to resources in content store (i128 resource Id)
 

## Editor Attributes

Attributes are used to express editor and serialization functionnality.

Example for potential attributes:
```rust
  #[readOnly(condition)]  // Property is readonly (with optional condition)
  #[defaultValue(value) // Default value
  #[hidden]               // Property is not visible in the Editor
  #[displayName(name)]    // Override Name of Property in the Editor
  #[tooltip(message)]  // Provide a toolTip 
  #[help(URL)]  // Provide a description / Help Url
  #[range(0,1)]   // Provide Editor validation for range limit
  #[category(Name)]   // Allow to group different properties under a collapsable property
  #[expandChildren]  // For array properties, auto expand children 
  #[propertyEditor(EditorName)]  // Override Editor Default property Editor
```

## Example

We need to explore what is doable in Rust, but here's a first draft of what a DataContainer code definition would look like:

```rust
#[derive(DataContainer)]
#[dataVersion(1)] // optional manual version
struct ObjectDefinition {

	#[range(1,8192)]
	#[defaultValue(256)]
	width : u32,

	#[defaultValue(Box)]
	shape_type : ShapeType,

	#[defaultValue("0x2b368fed00000000779bb4f05b3c9fc5213421341234")]
	#[hidden]
	resource_id : i128,
	
	#[readOnly(shape_type == Box)]
	#[toolTip("This is a conditional read-only property")]
	read_only_if_box : String,

	#[propertyEditor(ColorEditor)] 
	color : Vec4
}
```

