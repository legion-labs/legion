use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;
use crate::GenerationType;

pub(crate) fn generate_component(
    data_container_info: &DataContainerMetaInfo,
    gen_type: GenerationType,
) -> TokenStream {
    let type_identifier = format_ident!("{}", data_container_info.name);
    let type_name = &data_container_info.name;

    let tag_type = if gen_type == GenerationType::OfflineFormat {
        data_container_info.name.clone()
    } else {
        format!("Runtime_{}", &data_container_info.name)
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
