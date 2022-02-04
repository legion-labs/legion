use proc_macro2::TokenStream;
use quote::quote;

use crate::struct_meta_info::StructMetaInfo;
use crate::GenerationType;

pub(crate) fn generate_component(
    struct_info: &StructMetaInfo,
    gen_type: GenerationType,
) -> TokenStream {
    let type_identifier = &struct_info.name;
    let type_name = struct_info.name.to_string();

    let tag_type = if gen_type == GenerationType::OfflineFormat {
        struct_info.name.to_string()
    } else {
        format!("Runtime_{}", &struct_info.name)
    };

    // Registry a Factory for offline creation
    let factory_registry = if gen_type == GenerationType::OfflineFormat {
        quote! {
            inventory::submit! {
                lgn_data_runtime::ComponentFactory {
                    name : #type_name,
                    get_type_def : <#type_identifier as lgn_data_model::TypeReflection>::get_type_def
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[typetag::serde(name = #tag_type)]
        impl lgn_data_runtime::Component for #type_identifier {
            fn eq(&self, other: &dyn lgn_data_runtime::Component) -> bool {
                other
                .downcast_ref::<Self>()
                .map_or(false, |other| std::cmp::PartialEq::eq(self, other))
            }
        }

        #factory_registry
    }
}
