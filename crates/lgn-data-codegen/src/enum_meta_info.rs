use crate::{attributes::Attributes, member_meta_info::MemberMetaInfo};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Fields, ItemEnum};

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
}
