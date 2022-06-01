use std::collections::HashSet;

use petgraph::graphmap::DiGraphMap;

use super::{Model, ModelHandle, ModelObject};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CGenType {
    Native(NativeType),
    BitField(BitFieldType),
    Struct(StructType),
}

pub type CGenTypeHandle = ModelHandle<CGenType>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum NativeType {
    Float(usize),
    Uint(usize),
    Half(usize),
    Float4x4,
}

static FLOAT_TYPESTRINGS: [&str; 4] = ["Float1", "Float2", "Float3", "Float4"];
static UINT_TYPESTRINGS: [&str; 4] = ["Uint1", "Uint2", "Uint3", "Uint4"];
static HALF_TYPESTRINGS: [&str; 4] = ["Half1", "Half2", "Half3", "Half4"];

impl NativeType {
    pub fn name(&self) -> &str {
        match self {
            Self::Float(n) => {
                assert!(*n >= 1 && *n <= 4);
                FLOAT_TYPESTRINGS[n - 1]
            }
            Self::Uint(n) => {
                assert!(*n >= 1 && *n <= 4);
                UINT_TYPESTRINGS[n - 1]
            }
            Self::Half(n) => {
                assert!(*n >= 1 && *n <= 4);
                HALF_TYPESTRINGS[n - 1]
            }
            Self::Float4x4 => "Float4x4",
        }
    }
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
    pub fn is_native_type(&self) -> bool {
        matches!(self, CGenType::Native(_))
    }

    pub fn native_type(&self) -> &NativeType {
        if let CGenType::Native(e) = self {
            e
        } else {
            panic!("Invalid access");
        }
    }

    pub fn is_bitfield_type(&self) -> bool {
        matches!(self, CGenType::BitField(_))
    }

    pub fn bitfield_type(&self) -> &BitFieldType {
        if let CGenType::BitField(e) = self {
            e
        } else {
            panic!("Invalid access");
        }
    }

    pub fn is_struct_type(&self) -> bool {
        matches!(self, CGenType::Struct(_))
    }

    pub fn struct_type(&self) -> &StructType {
        if let CGenType::Struct(e) = self {
            e
        } else {
            panic!("Invalid access");
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CGenType::Native(e) => e.name(),
            CGenType::Struct(e) => e.name.as_str(),
            CGenType::BitField(e) => e.name.as_str(),
        }
    }

    pub fn get_type_dependencies(&self) -> HashSet<CGenTypeHandle> {
        let mut set = HashSet::new();
        match self {
            CGenType::Native(_) | CGenType::BitField(_) => (),
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitFieldType {
    pub name: String,
    pub values: Vec<String>,
}

impl BitFieldType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            values: Vec::new(),
        }
    }
}

impl ModelObject for CGenType {
    fn typename() -> &'static str {
        "CgenType"
    }
    fn name(&self) -> &str {
        self.name()
    }
}

pub type TypeGraph = DiGraphMap<u32, ()>;

pub fn build_type_graph(model: &Model) -> TypeGraph {
    let mut g = TypeGraph::new();

    for t in model.object_iter::<CGenType>() {
        g.add_node(t.id());
    }

    for t in model.object_iter::<CGenType>() {
        let deps = t.object().get_type_dependencies();
        for e in deps {
            g.add_edge(e.id(), t.id(), ());
        }
    }

    g
}

pub struct StructBuilder<'mdl> {
    mdl: &'mdl Model,
    product: StructType,
    names: HashSet<String>,
}

impl<'mdl> StructBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        StructBuilder {
            mdl,
            product: StructType::new(name),
            names: HashSet::new(),
        }
    }

    /// Add struct member
    ///
    /// # Errors
    /// todo
    pub fn add_member(mut self, name: &str, typ: &str, array_len: Option<u32>) -> Result<Self> {
        // check member uniqueness
        if self.names.contains(name) {
            return Err(anyhow!("Member '{}' already exists", name,));
        }
        self.names.insert(name.to_string());

        // check array_len validity
        if let Some(array_len) = array_len {
            if array_len == 0 {
                return Err(anyhow!("Member '{}' can't have a zero array_len", name,));
            }
        }

        // get cgen type and check its existence if necessary
        let ty_ref = self
            .mdl
            .get_object_handle::<CGenType>(typ)
            .context(anyhow!("Member '{}'  has an unknown type '{}'", name, typ))?;

        // done
        self.product
            .members
            .push(StructMember::new(name, ty_ref, array_len));
        Ok(self)
    }

    /// Build
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(self) -> Result<StructType> {
        Ok(self.product)
    }
}

pub struct BitFieldBuilder<'mdl> {
    _mdl: &'mdl Model,
    product: BitFieldType,
    values: HashSet<String>,
}

impl<'mdl> BitFieldBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        BitFieldBuilder {
            _mdl: mdl,
            product: BitFieldType::new(name),
            values: HashSet::new(),
        }
    }

    /// Add struct member
    ///
    /// # Errors
    /// todo
    pub fn add_value(mut self, value: &str) -> Result<Self> {
        // check member uniqueness
        if self.values.contains(value) {
            return Err(anyhow!("Value '{}' already exists", value));
        }

        if self.values.len() >= std::mem::size_of::<u32>() * 8 {
            return Err(anyhow!("Limit of 32 values has been reached"));
        }

        self.values.insert(value.to_string());

        // done
        self.product.values.push(value.to_owned());
        Ok(self)
    }

    /// Build
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(self) -> Result<BitFieldType> {
        Ok(self.product)
    }
}
