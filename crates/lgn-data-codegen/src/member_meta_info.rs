use quote::ToTokens;
use std::collections::HashSet;

use crate::attributes::{Attributes, OFFLINE_ONLY_ATTR, RESOURCE_TYPE_ATTR, RUNTIME_ONLY_ATTR};

#[derive(Debug)]
pub(crate) struct MemberMetaInfo {
    pub(crate) name: syn::Ident,
    pub(crate) type_path: syn::Path,
    pub(crate) attributes: Attributes,
    pub(crate) offline_imports: HashSet<syn::Path>,
    pub(crate) runtime_imports: HashSet<syn::Path>,
}

impl MemberMetaInfo {
    pub(crate) fn new(member_field: &syn::Field, type_path: syn::Path) -> Self {
        let mut member = Self {
            name: member_field.ident.as_ref().unwrap().clone(),
            type_path,
            attributes: Attributes::new(&member_field.attrs),
            offline_imports: HashSet::new(),
            runtime_imports: HashSet::new(),
        };

        // Add import if we have a ResourceType
        if member.attributes.values.contains_key(RESOURCE_TYPE_ATTR) {
            member
                .offline_imports
                .insert(syn::parse_str("lgn_data_offline::ResourcePathId").unwrap());
            member
                .runtime_imports
                .insert(syn::parse_str("lgn_data_runtime::Reference").unwrap());
        }
        member
    }

    pub(crate) fn is_option(&self) -> bool {
        !self.type_path.segments.is_empty() && self.type_path.segments[0].ident == "Option"
    }

    pub(crate) fn is_vec(&self) -> bool {
        !self.type_path.segments.is_empty() && self.type_path.segments[0].ident == "Vec"
    }

    pub(crate) fn get_type_name(&self) -> String {
        self.type_path.to_token_stream().to_string()
    }

    pub(crate) fn is_offline_only(&self) -> bool {
        self.attributes.values.contains_key(OFFLINE_ONLY_ATTR)
    }

    pub(crate) fn is_runtime_only(&self) -> bool {
        self.attributes.values.contains_key(RUNTIME_ONLY_ATTR)
    }

    pub(crate) fn get_runtime_type(&self) -> syn::Path {
        match self.get_type_name().as_str() {
            "Option < ResourcePathId >" => {
                let ty = if let Some(resource_type) = self.attributes.values.get(RESOURCE_TYPE_ATTR)
                {
                    format!("Option<{}ReferenceType>", resource_type)
                } else {
                    panic!("Option<ResourcePathId> must specify ResourceType in 'resource_type' attribute");
                };
                syn::parse_str(ty.as_str()).unwrap()
            }
            "Vec < ResourcePathId >" => {
                let ty = if let Some(resource_type) = self.attributes.values.get(RESOURCE_TYPE_ATTR)
                {
                    format!("Vec<{}ReferenceType>", resource_type)
                } else {
                    panic!("Vec<ResourcePathId> must specify ResourceType in 'resource_type' attribute");
                };
                syn::parse_str(ty.as_str()).unwrap()
            }
            _ => self.type_path.clone(), // Keep same
        }
    }
}
