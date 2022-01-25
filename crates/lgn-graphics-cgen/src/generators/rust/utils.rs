use crate::db::{CGenType, NativeType};

static FLOAT_TYPE_NAMES: [&str; 4] = ["Float1", "Float2", "Float3", "Float4"];
static UINT_TYPE_NAMES: [&str; 4] = ["Uint1", "Uint2", "Uint3", "Uint4"];
static HALF_TYPES_NAMES: [&str; 4] = ["Half1", "Half2", "Half3", "Half4"];

impl CGenType {
    pub(super) fn to_rust_name(&self) -> &str {
        let type_name = match self {
            CGenType::Native(e) => match e {
                NativeType::Float(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    FLOAT_TYPE_NAMES[n - 1]
                }
                NativeType::Uint(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    UINT_TYPE_NAMES[n - 1]
                }
                NativeType::Half(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    HALF_TYPES_NAMES[n - 1]
                }
                NativeType::Float4x4 => "Float4x4",
            },
            CGenType::Struct(e) => e.name.as_str(),
        };
        type_name
    }
}
