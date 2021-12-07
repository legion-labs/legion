use crate::model::{CGenType, Model, ModelKey, NativeType};

pub(super) fn get_rust_typestring<'a>(model: &Model, typekey: ModelKey) -> &str {
    let ty = model.get::<CGenType>(typekey).unwrap();
    let typestring = match ty {
        CGenType::Native(e) => match e {
            NativeType::Float1 => "Float1",
            NativeType::Float2 => "Float2",
            NativeType::Float3 => "Float3",
            NativeType::Float4 => "Float4",
            NativeType::Float4x4 => "Float4x4",
        },
        CGenType::Struct(e) => e.name.as_str(),
    };
    typestring
}
