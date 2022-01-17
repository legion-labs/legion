use crate::db::{CGenType, Model, NativeType, StructMember};

static FLOAT_TYPESTRINGS: [&str; 4] = ["float", "float2", "float3", "float4"];
static UINT_TYPESTRINGS: [&str; 4] = ["uint", "uint2", "uint3", "uint4"];
static HALF_TYPESTRINGS: [&str; 4] = ["half", "half2", "half3", "half4"];

pub(super) fn get_hlsl_typestring(ty: &CGenType) -> &str {
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
            NativeType::Float4x4 => "float4x4",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}

pub(super) fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_hlsl_typestring(member.ty_handle.get(model));

    if let Some(array_len) = member.array_len {
        format!("{} {}[{}];", typestring, member.name, array_len)
    } else {
        format!("{} {};", typestring, member.name)
    }
}
