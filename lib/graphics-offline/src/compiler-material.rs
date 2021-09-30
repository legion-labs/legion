use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::path_id_to_binary,
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::ResourceRegistryOptions;
use legion_data_runtime::AssetDescriptor;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        legion_graphics_offline::material::TYPE_ID.content(),
        legion_graphics_runtime::Material::TYPE.content(),
    ),
    compiler_hash_func: compiler_hash,
    compile_func: compile,
};

fn compiler_hash(
    code: &'static str,
    data: &'static str,
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
) -> CompilerHash {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    CompilerHash(hasher.finish())
}

fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistryOptions::new()
        .add_type(
            legion_graphics_offline::material::TYPE_ID,
            Box::new(legion_graphics_offline::material::MaterialProcessor {}),
        )
        .create_registry();

    let resource = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let resource = resource
        .get::<legion_graphics_offline::material::Material>(&resources)
        .unwrap();

    let compiled_asset = {
        let mut c: Vec<u8> = vec![];
        c.append(
            &mut path_id_to_binary(&resource.albedo, legion_graphics_runtime::Texture::TYPE)
                .to_le_bytes()
                .to_vec(),
        );
        c.append(
            &mut path_id_to_binary(&resource.normal, legion_graphics_runtime::Texture::TYPE)
                .to_le_bytes()
                .to_vec(),
        );
        c.append(
            &mut path_id_to_binary(&resource.roughness, legion_graphics_runtime::Texture::TYPE)
                .to_le_bytes()
                .to_vec(),
        );
        c.append(
            &mut path_id_to_binary(&resource.metalness, legion_graphics_runtime::Texture::TYPE)
                .to_le_bytes()
                .to_vec(),
        );
        c
    };

    let checksum = context
        .content_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: context.compile_path,
        checksum: checksum.into(),
        size: compiled_asset.len(),
    };

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
