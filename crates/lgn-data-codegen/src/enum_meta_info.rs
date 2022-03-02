use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Fields, ItemEnum, Path};

use crate::{attributes::Attributes, member_meta_info::MemberMetaInfo};

#[derive(Debug)]
pub(crate) struct EnumMetaInfo {
    pub name: syn::Ident,
    pub attributes: Attributes,
    pub variants: Vec<EnumVariantMetaInfo>,
}

impl EnumMetaInfo {
    pub(crate) fn new(item_enum: &ItemEnum) -> Self {
        let mut enum_meta_info = Self {
            name: item_enum.ident.clone(),
            attributes: Attributes::new(&item_enum.attrs),
            variants: Vec::new(),
        };

        for v in &item_enum.variants {
            enum_meta_info.variants.push(EnumVariantMetaInfo::new(v));
        }
        enum_meta_info
    }

    pub(crate) fn has_only_unit_variants(&self) -> bool {
        !self
            .variants
            .iter()
            .any(|variant| !variant.is_unit_variant())
    }

    pub(crate) fn offline_imports(&self) -> HashSet<&syn::Path> {
        let mut output = HashSet::new();
        for variant in &self.variants {
            variant.append_offline_imports(&mut output);
        }
        output
    }

    pub(crate) fn runtime_imports(&self) -> HashSet<&syn::Path> {
        let mut output = HashSet::new();
        for variant in &self.variants {
            variant.append_runtime_imports(&mut output);
        }
        output
    }
}

#[derive(Debug)]
pub(crate) struct EnumVariantMetaInfo {
    pub name: syn::Ident,
    pub discriminant: Option<TokenStream>,
    pub attributes: Attributes,
    pub members: Vec<MemberMetaInfo>,
}

impl EnumVariantMetaInfo {
    fn new(variant: &syn::Variant) -> Self {
        let members = match &variant.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .filter_map(MemberMetaInfo::new)
                .collect(),
            Fields::Unnamed(fields_unnamed) => fields_unnamed
                .unnamed
                .iter()
                .filter_map(MemberMetaInfo::new)
                .collect(),
            Fields::Unit => Vec::new(),
        };

        Self {
            name: variant.ident.clone(),
            discriminant: variant
                .discriminant
                .as_ref()
                .map(|(_e, expr)| expr.into_token_stream()),
            attributes: Attributes::new(&variant.attrs),
            members,
        }
    }

    fn is_unit_variant(&self) -> bool {
        self.members.is_empty()
    }

    pub(crate) fn append_offline_imports<'a>(&'a self, output: &mut HashSet<&'a Path>) {
        for member in &self.members {
            for import in &member.offline_imports {
                output.insert(import);
            }
        }
    }

    pub(crate) fn append_runtime_imports<'a>(&'a self, output: &mut HashSet<&'a Path>) {
        for member in &self.members {
            for import in &member.runtime_imports {
                output.insert(import);
            }
        }
    }
}
