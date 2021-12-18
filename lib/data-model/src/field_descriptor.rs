use crate::TypeDefinition;
/// Descriptor of a Reflected Field
pub struct FieldDescriptor {
    /// Name of the field
    pub field_name: String,
    /// Offset of the field in struct
    pub offset: usize,
    /// Type of the field
    pub field_type: TypeDefinition,
    /// Editor Group of the field
    pub group: String,
}
