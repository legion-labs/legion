use crate::db::{CGenType, NativeType};

static FLOAT_TYPESTRINGS: [&str; 4] = ["Float1", "Float2", "Float3", "Float4"];
static UINT_TYPESTRINGS: [&str; 4] = ["Uint1", "Uint2", "Uint3", "Uint4"];
static HALF_TYPESTRINGS: [&str; 4] = ["Half1", "Half2", "Half3", "Half4"];

pub(super) fn get_rust_typestring(ty: &CGenType) -> &str {
    let typestring = match ty {
        CGenType::Native(e) => match e {
            NativeType::Float(n) => {
                assert!(*n >= 1 && *n <= 4);
                FLOAT_TYPESTRINGS[n - 1]
            }
            NativeType::Uint(n) => {
                assert!(*n >= 1 && *n <= 4);
                UINT_TYPESTRINGS[n - 1]
            }
            NativeType::Half(n) => {
                assert!(*n >= 1 && *n <= 4);
                HALF_TYPESTRINGS[n - 1]
            }
            NativeType::Float4x4 => "Float4x4",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}
