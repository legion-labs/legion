use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;

fn generate_compile_resource(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let members_compile: Vec<TokenStream> = data_container_info
        .members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            match m.get_type_name().as_str() {
                "Option < ResourcePathId >" => quote! {
                    #member_ident :  offline.#member_ident.as_ref().map(|path| lgn_data_runtime::Reference::Passive(path.resource_id())),
                },
                "Vec < ResourcePathId >" => quote! {
                    #member_ident : offline.#member_ident.iter().map(|path| lgn_data_runtime::Reference::Passive(path.resource_id())).collect(),
                },
                "Vec < Box < dyn Component > >" => quote! {
                    #member_ident : Vec::new(),
                },
                _ => quote! {
                    #member_ident : offline.#member_ident.clone(),
                },
            }
        })
        .collect();

    quote! {
        #[allow(unused_variables,clippy::clone_on_copy)]
        fn compile_resource(offline: &OfflineType) -> RuntimeType {
            RuntimeType {
                #(#members_compile)*
            }
        }
    }
}

fn generate_extract_dependencies(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let extract_dependencies: Vec<TokenStream> = data_container_info
        .members
        .iter()
        .filter_map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            match m.get_type_name().as_str() {
                "Option < ResourcePathId >" => Some(quote! {
                    if let Some(value) = offline.#member_ident.as_ref() {
                        results.push(value.clone())
                    }
                }),
                "Vec < ResourcePathId >" => Some(quote! {
                    results.append(&offline.#member_ident);
                }),
                //"Vec < Box < dyn Component > >" => quote! {},
                _ => None,
            }
        })
        .collect();

    if extract_dependencies.is_empty() {
        quote! {
            fn extract_resource_dependencies(_offline: &OfflineType) -> Option<Vec<ResourcePathId>> {
                None
            }
        }
    } else {
        quote! {
            fn extract_resource_dependencies(offline: &OfflineType) -> Option<Vec<ResourcePathId>> {
                let mut results = Vec::new();
                #(#extract_dependencies)*
                Some(results)
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn generate(
    data_container_info: &DataContainerMetaInfo,
    crate_name: &syn::Ident,
) -> TokenStream {
    let type_name: syn::Ident = format_ident!("{}", data_container_info.name);

    let extract_depends_code = generate_extract_dependencies(data_container_info);
    let compile_resource_code = generate_compile_resource(data_container_info);

    let signature_hash = data_container_info.calculate_hash().to_string();

    quote! {

        use std::env;
        use lgn_data_compiler::{
            compiler_api::{
                CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
                DATA_BUILD_VERSION,
            },
            compiler_utils::hash_code_and_data,
        };

        use lgn_data_offline::{ResourcePathId, Transform};
        use lgn_data_runtime::{Resource};
        type OfflineType = #crate_name::offline::#type_name;
        type RuntimeType = #crate_name::runtime::#type_name;

        pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
            name: env!("CARGO_CRATE_NAME"),
            build_version: DATA_BUILD_VERSION,
            code_version: "1",
            data_version: #signature_hash,
            transform: &Transform::new(OfflineType::TYPE, RuntimeType::TYPE),
            compiler_hash_func: hash_code_and_data,
            compile_func: compile,
        };

        #extract_depends_code

        #compile_resource_code

        fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
            let resources = context.take_registry().add_loader::<OfflineType>().create();

            let offline_resource = resources.load_sync::<OfflineType>(context.source.resource_id());
            let offline_resource = offline_resource.get(&resources).unwrap();

            let runtime_resource = compile_resource(&offline_resource);
            let compiled_asset = bincode::serialize(&runtime_resource).unwrap();

            let resource_references = extract_resource_dependencies(&offline_resource);
            let resource_references: Vec<(ResourcePathId, ResourcePathId)> = resource_references.unwrap_or_default()
                .into_iter()
                .map(|res| (context.target_unnamed.clone(), res))
                .collect();

            let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

            Ok(CompilationOutput {
                compiled_resources: vec![asset],
                resource_references,
            })
        }
    }
}
