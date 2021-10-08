# DataContainer Format

Allow serialization and editing of data structures for runtime and editor process.
DataContainer are written as Rust struct, with custom attributes to extend functionnality.
Reflection is used for offline serialization, data compiltation and editor operations.

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
* Array types, Tuple types
* Reference to resources in content store (i128 resource Id)
 

## DataContainer Example

```rust
#[derive(DataContainer)]
pub struct TestEntity {
	// Default with string literal
	#[legion(default = "string literal", readonly, category = "Name")]
	test_string: String,

	// Default with Tuple()
	#[legion(default=(0.0,0.0,0.0), hidden)]
	pub test_position: Vec3,

	// Default with Constant value
	#[legion(default= Quat::IDENTITY, tooltip = "Rotation Tooltip")]
	pub test_rotation: Quat,

	// Default initialized from func call
	#[legion(default = func_hash_test(0x1234,"test"), transient)]
	pub test_transient: u64,

	// Default with bool constant
	#[legion(default = false)]
	test_bool: bool,

	// Default with Float constant
	#[legion(default = 32.32f32)]
	test_float32: f32,

	#[legion(default = 64.64f64, offline)]
	test_float64: f64,

	// Default with Enum
	#[legion(default = EnumTest::Value0, readonly)]
	pub test_enum: EnumTest,

	// Default with Integer constant
	#[legion(default = 123)]
	test_int: i32,

	// Default with Array
	#[legion(default=[0,1,2,3])]
	test_blob: Vec<u8>,
}
```

## DataContainer Attributes

DataContainer attributes can be used to change the code generation, the editor behavior and  serialization functionnality.

* #[legion(default = DefaultValueExpr)] attribute can be used to specify the default value of a field. This will be used to automatically generate the Rust 'Default' impl and serialization code. Any field at default Value will be skipped during Offline JSON serialization to allow easy upgrade and deprecation. Example of DefaultValueExpr:
	* "string literal" // String Literal
	* (1,1,1) // Tuple
	* [0,1,2,3] // Array
	* false // Constant 
	* 12.12f32 // Constant
	* Quat::IDENTITY // Constant identifier
	* Enum::Value  // Enum Value
	* hash_test("test") // Function Call
	

* #[legion(readonly)] attribute specify that the Editor should not allow the edition of the field.

* #[legion(hidden)] attribute specify that the field should be hidden in the Editor.

* #[legion(offline)] attribute specify that the field shouldn't be in the Runtime representation.

* #[legion(tooltip = "ToolTip message")] attribute specify the tool tip displayed when the user hover over the field name. 

* #[legion(category = "Rendering")] attribute specify the category of the field. In the Editor property inspector, fields will be grouped by category. 

* #[legion(transient)] attribute specify the field should not be serialize to the Offline Data. It is a field that's procedurally generated and shouldn't be save to disk.


