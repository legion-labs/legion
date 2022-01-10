use crate::model::{CGenType, NativeType, StructMember, Model};

pub(super) fn get_hlsl_typestring(ty: &CGenType) -> &str {
    let typestring = match ty {
        CGenType::Native(e) => match e {
            NativeType::Float1 => "float",
            NativeType::Float2 => "float2",
            NativeType::Float3 => "float3",
            NativeType::Float4 => "float4",
            NativeType::Float4x4 => "float4x4",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}

pub(super) fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_hlsl_typestring(member.ty_handle.get(model));

    if let Some(array_len) =  member.array_len {
        format!("{} {}[{}];", typestring, member.name, array_len)
    } else {
        format!("{} {};", typestring, member.name)
    }
}