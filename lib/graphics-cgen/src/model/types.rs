use strum::{EnumIter, IntoStaticStr};

use super::{model::ModelObjectRef, ModelObject};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CGenType {
    Native(NativeType),
    Struct(StructType),
}

pub type CGenTypeRef = ModelObjectRef<CGenType>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, EnumIter, IntoStaticStr)]
pub enum NativeType {
    Float1,
    Float2,
    Float3,
    Float4,
    Float4x4,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructMember {
    pub name: String,
    pub ty_ref: CGenTypeRef,
    pub array_len: Option<u32>,
}

impl StructMember {
    pub fn new(name: &str, ty_ref: CGenTypeRef, array_len: Option<u32>) -> Self {
        Self {
            name: name.to_owned(),
            ty_ref,
            array_len,
        }
    }
}

impl CGenType {
    pub fn struct_type(&self) -> &StructType {
        match self {
            CGenType::Struct(e) => e,
            CGenType::Native(_) => panic!("Invalid access"),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CGenType::Native(e) => e.into(),
            CGenType::Struct(e) => e.name.as_str(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructType {
    pub name: String,
    pub members: Vec<StructMember>,
}

impl StructType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }
}

impl ModelObject for CGenType {
    fn typename() -> &'static str {
        "CgenType"
    }
    fn name(&self) -> &str {
        match self {
            CGenType::Native(e) => e.into(),
            CGenType::Struct(e) => e.name.as_str(),
        }
    }
}
