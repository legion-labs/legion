use std::hash::{Hash, Hasher};

use crate::attributes::Attributes;
use crate::member_meta_info::MemberMetaInfo;
use lgn_utils::DefaultHasher;
use std::collections::HashSet;
use syn::{parse_quote, Ident, ItemStruct, Type};

#[derive(Debug)]
pub(crate) struct StructMetaInfo {
    pub(crate) name: Ident,
    pub(crate) need_life_time: bool,
    pub(crate) attributes: Attributes,
    pub(crate) members: Vec<MemberMetaInfo>,
    pub(crate) is_resource: bool,
    pub(crate) is_component: bool,
}

// Integrate a field 'meta' as the first field in each generated struct.
fn compute_meta(ident: &Ident) -> Vec<MemberMetaInfo> {
    let meta: ItemStruct = parse_quote!(
        struct MetaContainer {
            #[legion(offline_only)]
            #[legion(default=Metadata::new(ResourcePathName::default(), <#ident as lgn_data_runtime::ResourceDescriptor>::TYPENAME, <#ident as lgn_data_runtime::ResourceDescriptor>::TYPE))]
            meta: Metadata,
        }
    );

    meta.fields
        .iter()
        .filter_map(|field| {
            if let Type::Path(type_path) = &field.ty {
                Some(MemberMetaInfo::new(field, type_path.path.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<MemberMetaInfo>>()
}

impl StructMetaInfo {
    pub(crate) fn new(item_struct: &ItemStruct) -> Self {
        let is_resource = item_struct
            .attrs
            .iter()
            .any(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "resource");
        let is_component = item_struct.attrs.iter().any(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == "component"
        });

        let mut members = if is_resource {
            compute_meta(&item_struct.ident)
        } else {
            vec![]
        };
        members.extend(item_struct.fields.iter().filter_map(|field| {
            if let Type::Path(type_path) = &field.ty {
                Some(MemberMetaInfo::new(field, type_path.path.clone()))
            } else {
                None
            }
        }));

        Self {
            name: item_struct.ident.clone(),
            need_life_time: false,
            attributes: Attributes::new(&item_struct.attrs),
            is_resource,
            is_component,
            members,
        }
    }

    pub(crate) fn need_life_time(&self) -> bool {
        self.need_life_time
    }

    pub(crate) fn offline_imports(&self) -> HashSet<&syn::Path> {
        let mut output = HashSet::new();
        for member in &self.members {
            for import in &member.offline_imports {
                output.insert(import);
            }
        }
        output
    }

    pub(crate) fn runtime_imports(&self) -> HashSet<&syn::Path> {
        let mut output = HashSet::new();
        for member in &self.members {
            for import in &member.runtime_imports {
                output.insert(import);
            }
        }
        output
    }

    pub(crate) fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.members.iter().for_each(|m| {
            m.name.hash(&mut hasher);
            m.get_type_name().hash(&mut hasher);

            m.attributes.values.iter().for_each(|(k, v)| {
                k.hash(&mut hasher);
                v.hash(&mut hasher);
            });

            if let Some(default_lit) = &m.attributes.default_literal {
                default_lit.to_string().hash(&mut hasher);
            }
        });

        hasher.finish()
    }
}
