use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;
use crate::GenerationType;

pub(crate) fn generate_component(
    data_container_info: &DataContainerMetaInfo,
    gen_type: GenerationType,
) -> TokenStream {
    let type_identifier = format_ident!("{}", data_container_info.name);

    let tag_type = if gen_type == GenerationType::OfflineFormat {
        data_container_info.name.clone()
    } else {
        format!("Runtime_{}", &data_container_info.name)
    };
    quote! {
        #[typetag::serde(name = #tag_type)]
        impl lgn_data_runtime::Component for #type_identifier {}
    }
}
