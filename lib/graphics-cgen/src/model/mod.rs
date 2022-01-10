use strum::IntoEnumIterator;

mod model;
pub use model::Model;
pub use model::ModelHandle;
pub use model::ModelObject;

mod types;
pub use types::CGenType;
pub use types::CGenTypeHandle;
pub use types::NativeType;
pub use types::StructMember;
pub use types::StructType;
pub use types::build_type_graph;

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
    for native_type in NativeType::iter() {
        model
            .add(native_type.into(), CGenType::Native(native_type))
            .unwrap();
    }
    model
}
