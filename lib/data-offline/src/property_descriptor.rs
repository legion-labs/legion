/// Struct to describe a Property for Editor representation
pub struct PropertyDescriptor {
    /// Name of the Property
    pub name: &'static str,
    /// Type of the Property
    pub type_name: &'static str,
    /// Default value of the property
    pub default_value: Vec<u8>,
    /// Current value of the property
    pub value: Vec<u8>,
    /// Group of the property
    pub group: String,
}
