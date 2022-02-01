use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::struct_meta_info::StructMetaInfo;

pub(crate) fn generate(
    data_container_info: &StructMetaInfo,
    crate_name: &syn::Ident,
) -> TokenStream {
    let type_name: syn::Ident = format_ident!("{}", data_container_info.name);
    let signature_hash = data_container_info.calculate_hash().to_string();

    quote! {
        use std::env;
        use lgn_data_compiler::{
            compiler_api::{
                CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
                DATA_BUILD_VERSION,
            },
            compiler_utils::{hash_code_and_data},
            compiler_reflection::reflection_compile
        };

        use lgn_data_offline::{ResourcePathId, Transform};
        use lgn_data_runtime::{AssetRegistryOptions, Resource};
        type OfflineType = #crate_name::offline::#type_name;
        type RuntimeType = #crate_name::runtime::#type_name;

        pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
            name: env!("CARGO_CRATE_NAME"),
            build_version: DATA_BUILD_VERSION,
            code_version: "1",
            data_version: #signature_hash,
            transform: &Transform::new(OfflineType::TYPE, RuntimeType::TYPE),
            init_func: init,
            compiler_hash_func: hash_code_and_data,
            compile_func: compile,
        };

        fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
            registry.add_loader::<OfflineType>()
        }

        fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
            let resources = context.registry();

            let offline_resource = resources.load_sync::<OfflineType>(context.source.resource_id());
            if let Some(err) = resources.retrieve_err(offline_resource.id()) {
                return Err(CompilerError::CompilationError(err.to_string()));
            }
            let offline_resource = offline_resource
                .get(&resources)
                .ok_or_else(|| CompilerError::CompilationError(format!("Failed to retrieve resource '{}'", context.source.resource_id())))?;

            let offline_resource : &OfflineType = &offline_resource;
            let mut runtime_resource = RuntimeType::default();
            let (compiled_asset, resource_references) = reflection_compile(offline_resource, &mut runtime_resource)?;
            let resource_references: Vec<(ResourcePathId, ResourcePathId)> = resource_references
                .unwrap_or_default()
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
