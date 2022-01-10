mod model;
pub use model::Model;
pub use model::ModelHandle;
pub use model::ModelObject;

mod types;
pub use types::build_type_graph;
pub use types::CGenType;
pub use types::CGenTypeHandle;
pub use types::NativeType;
pub use types::StructMember;
pub use types::StructType;

mod descriptor_set;
pub use descriptor_set::ConstantBufferDef;
pub use descriptor_set::Descriptor;
pub use descriptor_set::DescriptorDef;
pub use descriptor_set::DescriptorSet;
pub use descriptor_set::DescriptorSetHandle;
pub use descriptor_set::StructuredBufferDef;
pub use descriptor_set::TextureDef;

mod pipeline_layout;
pub use pipeline_layout::PipelineLayout;
pub use pipeline_layout::PipelineLayoutContent;
pub use pipeline_layout::PipelineLayoutHandle;

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
        NativeType::Float16(1),
        NativeType::Float16(2),
        NativeType::Float16(3),
        NativeType::Float16(4),
        NativeType::Float4x4,
    ];

    native_types.iter().for_each(|native_type| {
        model
            .add(native_type.name(), CGenType::Native(*native_type))
            .unwrap();
    });

    model
}
