use std::collections::HashSet;

use petgraph::graphmap::DiGraphMap;
use strum::{EnumIter, IntoStaticStr};

use super::{Model, ModelHandle, ModelObject};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CGenType {
    Native(NativeType),
    Struct(StructType),
}

pub type CGenTypeHandle = ModelHandle<CGenType>;

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
    pub ty_handle: CGenTypeHandle,
    pub array_len: Option<u32>,
}

impl StructMember {
    pub fn new(name: &str, ty_ref: CGenTypeHandle, array_len: Option<u32>) -> Self {
        Self {
            name: name.to_owned(),
            ty_handle: ty_ref,
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

    pub fn get_type_dependencies(&self) -> HashSet<CGenTypeHandle> {
        let mut set = HashSet::new();
        match self {
            CGenType::Native(_) => {}
            CGenType::Struct(struct_ty) => {
                for mb in &struct_ty.members {
                    set.insert(mb.ty_handle);
                }
            }
        }
        set
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

pub type TypeGraph = DiGraphMap<u32, ()>;

pub fn build_type_graph(model: &Model) -> TypeGraph {
    let mut g = TypeGraph::new();

    for t in model.object_iter::<CGenType>() {
        g.add_node(t.id());
    }

    for n in model.object_iter::<CGenType>() {
        let deps = n.object().get_type_dependencies();
        for e in deps {
            g.add_edge(e.id(), n.id(), ());
        }
    }

    g
}
