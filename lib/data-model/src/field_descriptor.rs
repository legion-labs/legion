use crate::TypeDefinition;
use std::collections::HashMap;
/// Descriptor of a Reflected Field
pub struct FieldDescriptor {
    /// Name of the field
    pub field_name: String,
    /// Offset of the field in struct
    pub offset: usize,
    /// Type of the field
    pub field_type: TypeDefinition,
    /// List of Attributes for the Field
    pub attributes: HashMap<String, String>,
}
