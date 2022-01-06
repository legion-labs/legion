mod model;
pub use model::Model;
pub use model::ModelObject;
pub use model::ModelObjectRef;

mod types;
use strum::IntoEnumIterator;
pub use types::CGenType;
pub use types::CGenTypeRef;
pub use types::NativeType;
pub use types::StructMember;
pub use types::StructType;

mod descriptor_set;
pub use descriptor_set::ConstantBufferDef;
pub use descriptor_set::Descriptor;
pub use descriptor_set::DescriptorDef;
pub use descriptor_set::DescriptorSet;
pub use descriptor_set::DescriptorSetRef;
pub use descriptor_set::StructuredBufferDef;
pub use descriptor_set::TextureDef;

mod pipeline_layout;
pub use pipeline_layout::PipelineLayout;
pub use pipeline_layout::PipelineLayoutContent;
pub use pipeline_layout::PipelineLayoutRef;

pub fn create() -> Model {
    let mut model = Model::new();
    for native_type in NativeType::iter() {
        model
            .add(native_type.into(), CGenType::Native(native_type))
            .unwrap();
    }
    model
}
