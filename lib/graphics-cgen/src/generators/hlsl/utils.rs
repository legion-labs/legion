use crate::model::{CGenType, Model, ModelKey, NativeType};

pub(super) fn get_hlsl_typestring<'a>(model: &Model, typekey: ModelKey) -> &str {
    let ty = model.get::<CGenType>(typekey).unwrap();
    let typestring = match ty {
        CGenType::Native(e) => match e {
            NativeType::Float1 => "float",
            NativeType::Float2 => "float2",
            NativeType::Float3 => "float3",
            NativeType::Float4 => "float4",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}