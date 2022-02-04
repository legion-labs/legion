use crate::attributes::Attributes;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ItemEnum;

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
}

impl EnumVariantMetaInfo {
    fn new(variant: &syn::Variant) -> Self {
        Self {
            name: variant.ident.clone(),
            discriminant: variant
                .discriminant
                .as_ref()
                .map(|(_e, expr)| expr.into_token_stream()),
            attributes: Attributes::new(&variant.attrs),
        }
    }
}
