mod model;
pub(crate) use model::*;

mod types;
pub(crate) use types::*;

mod descriptor_set;
pub(crate) use descriptor_set::*;

mod pipeline_layout;
pub(crate) use pipeline_layout::*;

mod shader;
pub(crate) use shader::*;

pub fn create() -> Model {
    let mut model = Model::new();

    let native_types = [
        NativeType::Float(1),
        NativeType::Float(2),
        NativeType::Float(3),
        NativeType::Float(4),
        NativeType::Uint(1),
        NativeType::Uint(2),
        NativeType::Uint(3),
        NativeType::Uint(4),
        NativeType::Half(1),
        NativeType::Half(2),
        NativeType::Half(3),
        NativeType::Half(4),
        NativeType::Float4x4,
    ];

    for native_type in &native_types {
        model
            .add(native_type.name(), CGenType::Native(*native_type))
            .unwrap();
    }

    model
}
