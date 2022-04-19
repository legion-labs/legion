use std::hash::{Hash, Hasher};

use crate::member_meta_info::MemberMetaInfo;
use crate::{attributes::Attributes, GenerationType};
use lgn_utils::DefaultHasher;
use std::collections::HashSet;
use syn::{parse_quote, Ident, ItemStruct, Type};

#[derive(Debug)]
pub(crate) struct StructMetaInfo {
    pub(crate) parent_crate: Ident,
    pub(crate) name: Ident,
    pub(crate) attributes: Attributes,
    pub(crate) members: Vec<MemberMetaInfo>,
    pub(crate) is_resource: bool,
    pub(crate) is_component: bool,

    /// If set, tells to which generation this struct applies to. Otherwise, support all generations.
    pub(crate) only_generation: Option<GenerationType>,
}

// Integrate a field 'meta' as the first field in each generated struct.
fn integrate_meta(struct_name: &Ident, crate_name: &Ident) -> Vec<MemberMetaInfo> {
    // Skip ourselves if it's the Metadata struct.
    if *struct_name == "Metadata" {
        return vec![];
    }

    let meta: ItemStruct = parse_quote!(
        struct MetaContainer {
            #[legion(offline_only)]
            #[legion(default=Metadata::new_default::<Self>())]
            meta: Metadata,
        }
    );

    meta.fields
        .iter()
        .filter_map(|field| match &field.ty {
            Type::Path(type_path) => {
                let mut member = MemberMetaInfo::new(field, type_path.path.clone());
                // The Metadata is already in scope when in the crate below.
                if crate_name != "lgn_data_offline" {
                    member
                        .offline_imports
                        .insert(syn::parse_str("lgn_data_offline::offline::Metadata").unwrap());
                }
                Some(member)
            }
            _ => None,
        })
        .collect::<Vec<MemberMetaInfo>>()
}

impl StructMetaInfo {
    pub(crate) fn new(item_struct: &ItemStruct, crate_name: &Ident) -> Self {
        let is_resource = item_struct
            .attrs
            .iter()
            .any(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "resource");
        let is_component = item_struct.attrs.iter().any(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == "component"
        });

        let attrs = Attributes::new(&item_struct.attrs);
        let only_generation = if attrs
            .values
            .iter()
            .any(|v| v.0 == crate::attributes::OFFLINE_ONLY_ATTR)
        {
            Some(GenerationType::OfflineFormat)
        } else if attrs
            .values
            .iter()
            .any(|v| v.0 == crate::attributes::RUNTIME_ONLY_ATTR)
        {
            Some(GenerationType::RuntimeFormat)
        } else {
            None
        };

        let mut members = if is_resource {
            integrate_meta(&item_struct.ident, crate_name)
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
            parent_crate: crate_name.clone(),
            name: item_struct.ident.clone(),
            attributes: Attributes::new(&item_struct.attrs),
            is_resource,
            is_component,
            members,
            only_generation,
        }
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

    pub(crate) fn should_skip(&self, gen_type: GenerationType) -> bool {
        if let Some(g) = self.only_generation {
            if g != gen_type {
                return true;
            }
        }
        false
    }
}
