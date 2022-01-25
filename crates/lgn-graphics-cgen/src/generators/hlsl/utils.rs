use crate::db::{CGenType, Model, NativeType, StructMember};

static FLOAT_TYPE_NAMES: [&str; 4] = ["float", "float2", "float3", "float4"];
static UINT_TYPE_NAMES: [&str; 4] = ["uint", "uint2", "uint3", "uint4"];
static HALF_TYPE_NAMES: [&str; 4] = ["half", "half2", "half3", "half4"];

impl CGenType {
    pub(super) fn to_hlsl_name(&self) -> &str {
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
                    HALF_TYPE_NAMES[n - 1]
                }
                NativeType::Float4x4 => "float4x4",
            },
            CGenType::Struct(e) => e.name.as_str(),
        };
        type_name
    }
}

pub(super) fn member_declaration(model: &Model, member: &StructMember) -> String {
    let hlsl_name = member.ty_handle.get(model).to_hlsl_name();

    if let Some(array_len) = member.array_len {
        format!("{} {}[{}];", hlsl_name, member.name, array_len)
    } else {
        format!("{} {};", hlsl_name, member.name)
    }
}
