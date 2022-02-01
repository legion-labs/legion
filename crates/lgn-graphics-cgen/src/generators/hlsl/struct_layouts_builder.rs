use anyhow::{anyhow, Result};
use petgraph::{algo::toposort, EdgeDirection::Outgoing};
use spirv_reflect::types::ReflectBlockVariable;

use crate::{
    db::{build_type_graph, CGenType, DescriptorSet, Model, PipelineLayout, StructType},
    struct_layout::{StructLayout, StructLayouts, StructMemberLayout},
};

use super::utils::member_declaration;

impl StructLayout {
    pub fn from_spirv_reflect(ty: &StructType, block_var: &ReflectBlockVariable) -> Self {
        let padded_size = block_var.padded_size;

        assert_eq!(ty.members.len(), block_var.members.len());

        let mut members = Vec::with_capacity(ty.members.len());

        for i in 0..ty.members.len() {
            let spirv_member = &block_var.members[i];
            members.push(StructMemberLayout {
                offset: spirv_member.offset,
                padded_size: spirv_member.padded_size,
                array_stride: spirv_member.array.stride,
            });
        }

        Self {
            padded_size,
            members,
        }
    }
}

struct TypeUsage {
    cb: Vec<bool>,
    sb: Vec<bool>,
}

impl TypeUsage {
    fn new(size: usize) -> Self {
        Self {
            cb: (0..size).map(|_| false).collect(),
            sb: (0..size).map(|_| false).collect(),
        }
    }

    fn set_used_as_cb(&mut self, id: u32) {
        self.cb[id as usize] = true;
    }

    fn set_used_as_sb(&mut self, id: u32) {
        self.sb[id as usize] = true;
    }

    fn used_as_cb(&self, id: u32) -> bool {
        self.cb[id as usize]
    }

    fn used_as_sb(&self, id: u32) -> bool {
        self.sb[id as usize]
    }

    fn used_as_cb_or_sb(&self, id: u32) -> bool {
        self.used_as_cb(id) || self.used_as_sb(id)
    }

    fn used_as_cb_and_sb(&self, id: u32) -> bool {
        self.used_as_cb(id) && self.used_as_sb(id)
    }
}

//
/// # Errors
///
/// Will return `Err` if there is a cyclic dependency or if there are some type
/// used with different incompatible layouts.
///

pub fn run(model: &Model) -> Result<StructLayouts> {
    // Compute type dependency graph
    let graph = build_type_graph(model);

    // Topo sort
    // After this line, we are sure there is no cycles
    let ordered = toposort(&graph, None).map_err(|err| {
        anyhow!(
            "Cycle detected during type resolution pass (type: {}).",
            model.get_from_id::<CGenType>(err.node_id()).unwrap().name()
        )
    })?;

    // Compute external requirements from DescriptorSet and PushConstants
    let mut ty_requirements = TypeUsage::new(model.size::<CGenType>());

    for ds in model.object_iter::<DescriptorSet>() {
        let ds = ds.object();
        for d in &ds.descriptors {
            match &d.def {
                crate::db::DescriptorDef::ConstantBuffer(ty) => {
                    ty_requirements.set_used_as_cb(ty.ty_handle.id());
                }
                crate::db::DescriptorDef::StructuredBuffer(ty)
                | crate::db::DescriptorDef::RWStructuredBuffer(ty) => {
                    ty_requirements.set_used_as_sb(ty.ty_handle.id());
                }

                crate::db::DescriptorDef::Sampler
                | crate::db::DescriptorDef::ByteAddressBuffer
                | crate::db::DescriptorDef::RWByteAddressBuffer
                | crate::db::DescriptorDef::Texture2D(_)
                | crate::db::DescriptorDef::RWTexture2D(_)
                | crate::db::DescriptorDef::Texture3D(_)
                | crate::db::DescriptorDef::RWTexture3D(_)
                | crate::db::DescriptorDef::Texture2DArray(_)
                | crate::db::DescriptorDef::RWTexture2DArray(_)
                | crate::db::DescriptorDef::TextureCube(_)
                | crate::db::DescriptorDef::TextureCubeArray(_) => (),
            }
        }
    }

    for pl in model.object_iter::<PipelineLayout>() {
        let pl = pl.object();
        if let Some(pc) = &pl.push_constant {
            ty_requirements.set_used_as_sb(pc.id());
        }
    }

    // All the types not referenced by a ConstBuffer or a [RW]StructuredBuffer are potentially
    // used by a [RW]ByteAdressBuffer. Mark them as 'sb'.
    for ty in model.object_iter::<CGenType>() {
        match ty.object() {
            CGenType::Native(_) => (),
            CGenType::Struct(_) => {
                if !ty_requirements.used_as_cb_or_sb(ty.id()) {
                    ty_requirements.set_used_as_sb(ty.id());
                }
            }
        }
    }

    // Propagate requirements in topological order
    for id in &ordered {
        let id = *id;
        for n in graph.neighbors_directed(id, Outgoing) {
            if ty_requirements.used_as_cb(id) {
                ty_requirements.set_used_as_cb(n);
            }
            if ty_requirements.used_as_sb(id) {
                ty_requirements.set_used_as_sb(n);
            }
        }
    }

    // Generate a stub shader to extract our struct layouts
    let mut text = String::new();

    for id in &ordered {
        let id = *id;
        let ty = model.get_from_id::<CGenType>(id).unwrap();
        match ty {
            CGenType::Native(_) => (),
            CGenType::Struct(struct_ty) => {
                text.push_str(&format!("struct {} {{\n", struct_ty.name));
                for m in &struct_ty.members {
                    text.push_str(&member_declaration(model, m));
                    text.push('\n');
                }
                text.push_str("};\n");
            }
        }
    }

    for id in &ordered {
        let id = *id;
        if ty_requirements.used_as_cb_or_sb(id) {
            let ty = model.get_from_id::<CGenType>(id).unwrap();
            match ty {
                CGenType::Native(_) => (),
                CGenType::Struct(struct_ty) => {
                    text.push_str(&format!("ConstantBuffer<{}> cb_{}; \n", struct_ty.name, id));
                    text.push_str(&format!(
                        "StructuredBuffer<{}> sb_{}; \n",
                        struct_ty.name, id
                    ));
                }
            }
        }
    }

    text.push_str("void main() {}");

    let spirv_bytecode = hassle_rs::compile_hlsl(
        "cgen_mem_layouts.mem",
        &text,
        "main",
        "vs_6_2",
        &[
            "-Od",
            "-spirv",
            "-fspv-target-env=vulkan1.1",
            "-enable-16bit-types",
            "-HV 2021",
        ],
        &[],
    )?;

    let shader_mod = spirv_reflect::create_shader_module(&spirv_bytecode).unwrap();
    let mut layouts = StructLayouts::new();

    for binding in &shader_mod.enumerate_descriptor_bindings(None).unwrap() {
        let dt = &binding.name[0..2]; // sb_ | cb_
        let id = binding.name[3..].parse().unwrap(); // type id
        let ty = model.get_from_id::<CGenType>(id).unwrap();
        let block_var = match dt {
            "cb" => &binding.block,
            "sb" => &binding.block.members[0],
            _ => unreachable!(),
        };
        let new_layout = StructLayout::from_spirv_reflect(ty.struct_type(), block_var);
        if let Some(existing_layout) = layouts.get(id) {
            if existing_layout != &new_layout && ty_requirements.used_as_cb_and_sb(id) {
                println!("{:?}", existing_layout);
                println!("{:?}", new_layout);

                return Err(anyhow!(
                    "Type {} is used with different memory layouts.",
                    ty.name()
                ));
            }
        } else {
            match dt {
                "cb" => {
                    if ty_requirements.used_as_cb(id) {
                        layouts.insert(id, new_layout);
                    }
                }
                "sb" => {
                    if ty_requirements.used_as_sb(id) {
                        layouts.insert(id, new_layout);
                    }
                }
                _ => unreachable!(),
            };
        }
    }

    Ok(layouts)
}
