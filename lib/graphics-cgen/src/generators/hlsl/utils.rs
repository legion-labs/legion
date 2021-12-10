use crate::model::{CGenType, Model, ModelObjectId, NativeType};

pub(super) fn get_hlsl_typestring<'a>(model: &Model, object_id: ModelObjectId) -> &str {
    let ty = model.get_from_objectid::<CGenType>(object_id).unwrap();
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
