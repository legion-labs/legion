mod model;
pub use model::{Model, ModelHandle, ModelObject};

mod types;
pub use types::{build_type_graph, CGenType, CGenTypeHandle, NativeType, StructMember, StructType};

mod descriptor_set;
pub use descriptor_set::{
    ConstantBufferDef, Descriptor, DescriptorDef, DescriptorSet, DescriptorSetHandle,
    StructuredBufferDef, TextureDef,
};

mod pipeline_layout;
pub use pipeline_layout::{PipelineLayout, PipelineLayoutHandle};

mod builder;
pub use builder::*;

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
