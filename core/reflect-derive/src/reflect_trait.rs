use legion_macro_utils::LegionManifest;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Attribute, Ident, ItemTrait, Token};

pub struct TraitInfo {
    item_trait: ItemTrait,
}

impl Parse for TraitInfo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![pub]) || lookahead.peek(Token![trait]) {
            let mut item_trait: ItemTrait = input.parse()?;
            item_trait.attrs = attrs;
            Ok(TraitInfo { item_trait })
        } else {
            Err(lookahead.error())
        }
    }
}

pub fn reflect_trait(_args: TokenStream, input: TokenStream) -> TokenStream {
    let trait_info = parse_macro_input!(input as TraitInfo);
    let item_trait = &trait_info.item_trait;
    let trait_ident = &item_trait.ident;
    let reflect_trait_ident =
        Ident::new(&format!("Reflect{}", item_trait.ident), Span::call_site());
    let legion_reflect_path = LegionManifest::default().get_path("legion_reflect");
    TokenStream::from(quote! {
        #item_trait

        #[derive(Clone)]
        pub struct #reflect_trait_ident {
            get_func: fn(&dyn #legion_reflect_path::Reflect) -> Option<&dyn #trait_ident>,
            get_mut_func: fn(&mut dyn #legion_reflect_path::Reflect) -> Option<&mut dyn #trait_ident>,
        }

        impl #reflect_trait_ident {
            fn get<'a>(&self, reflect_value: &'a dyn #legion_reflect_path::Reflect) -> Option<&'a dyn #trait_ident> {
                (self.get_func)(reflect_value)
            }

            fn get_mut<'a>(&self, reflect_value: &'a mut dyn #legion_reflect_path::Reflect) -> Option<&'a mut dyn #trait_ident> {
                (self.get_mut_func)(reflect_value)
            }
        }

        impl<T: #trait_ident + #legion_reflect_path::Reflect> #legion_reflect_path::FromType<T> for #reflect_trait_ident {
            fn from_type() -> Self {
                Self {
                    get_func: |reflect_value| {
                        reflect_value.downcast_ref::<T>().map(|value| value as &dyn #trait_ident)
                    },
                    get_mut_func: |reflect_value| {
                        reflect_value.downcast_mut::<T>().map(|value| value as &mut dyn #trait_ident)
                    }
                }
            }
        }
    })
}
