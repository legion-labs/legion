use crate::model::{CGenType, Model, ModelKey, NativeType};

pub(super) fn get_rust_typestring<'a>(model: &Model, typekey: ModelKey) -> &str {
    let ty = model.get::<CGenType>(typekey).unwrap();
    let typestring = match ty {
        CGenType::Native(e) => match e {
            NativeType::Float1 => "f32",
            NativeType::Float2 => "f32",
            NativeType::Float3 => "f32",
            NativeType::Float4 => "f32",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}
