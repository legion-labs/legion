//! Compiler Node groups data compilers and provides utilities required for data compilation.
//!
//! registry defines an interface between different compiler types
//! (executable, in-process, etc).

use core::fmt;
use std::sync::Arc;

mod binary_stub;
mod inproc_stub;
pub(crate) mod remote_data_executor;
mod remote_stub;

mod compiler_registry;
pub use compiler_registry::*;

use lgn_data_runtime::AssetRegistry;

/// A group of compilers with compilation utilities.
pub struct CompilerNode {
    compilers: CompilerRegistry,
    registry: Arc<AssetRegistry>,
}

impl CompilerNode {
    /// Creates a new `CompilerNode`.
    pub fn new(compilers: CompilerRegistry, registry: Arc<AssetRegistry>) -> Self {
        Self {
            compilers,
            registry,
        }
    }

    /// Access to all compilers.
    pub fn compilers(&self) -> &CompilerRegistry {
        &self.compilers
    }

    /// Access to `AssetRegistry` shared between compilers.
    pub fn registry(&self) -> Arc<AssetRegistry> {
        self.registry.clone()
    }
}

impl fmt::Debug for CompilerNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerNode")
            .field("compilers", &self.compilers)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lgn_content_store::{Checksum, ContentStoreAddr};
    use lgn_data_offline::{ResourcePathId, Transform};
    use lgn_data_runtime::{
        AssetRegistryOptions, Resource, ResourceId, ResourceType, ResourceTypeAndId,
    };

    use super::CompilerRegistryOptions;
    use crate::{
        compiler_api::{CompilationEnv, CompilationOutput, CompilerDescriptor},
        CompiledResource, CompilerHash, Locale, Platform, Target,
    };

    const TEST_TRANSFORM: Transform =
        Transform::new(ResourceType::new(b"input"), ResourceType::new(b"output"));
    const TEST_COMPILER: CompilerDescriptor = CompilerDescriptor {
        name: "test_name",
        build_version: "build0",
        code_version: "code0",
        data_version: "data0",
        transform: &TEST_TRANSFORM,
        init_func: |options| options,
        compiler_hash_func: |_code, _data, _env| CompilerHash(7),
        compile_func: |ctx| {
            Ok(CompilationOutput {
                compiled_resources: vec![CompiledResource {
                    path: ctx.target_unnamed,
                    checksum: Checksum::from([7u8; 32]),
                    size: 7,
                }],
                resource_references: vec![],
            })
        },
    };

    #[test]
    fn binary() {
        let target_dir = std::env::current_exe().ok().map_or_else(
            || panic!("cannot find test directory"),
            |mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            },
        );

        let registry = CompilerRegistryOptions::local_compilers(target_dir).create();

        let env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let source = ResourceTypeAndId {
            kind: text_resource::TextResource::TYPE,
            id: ResourceId::new(),
        };
        let destination = ResourcePathId::from(source).push(integer_asset::IntegerAsset::TYPE);

        let transform = Transform::new(source.kind, destination.content_type());

        let (compiler, transform) = registry.find_compiler(transform).expect("valid compiler");
        let _ = compiler.compiler_hash(transform, &env).expect("valid hash");
    }

    #[test]
    fn in_proc() {
        let registry = CompilerRegistryOptions::default()
            .add_compiler(&TEST_COMPILER)
            .create();

        let env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let (compiler, transform) = registry.find_compiler(TEST_TRANSFORM).expect("a compiler");
        let hash = compiler.compiler_hash(transform, &env).expect("valid hash");
        assert_eq!(hash, CompilerHash(7));

        let source = ResourceTypeAndId {
            kind: ResourceType::new(b"input"),
            id: ResourceId::new(),
        };
        let cas = ContentStoreAddr::from(".");
        let proj_dir = PathBuf::from(".");
        let compile_path = ResourcePathId::from(source).push(ResourceType::new(b"output"));

        // testing successful compilation
        {
            let registry = AssetRegistryOptions::new().create();
            let output = compiler
                .compile(
                    compile_path.clone(),
                    &[],
                    &[],
                    registry,
                    cas,
                    &proj_dir,
                    &env,
                )
                .expect("valid output");

            assert_eq!(output.compiled_resources.len(), 1);
            assert_eq!(output.compiled_resources[0].path, compile_path);
            assert_eq!(
                output.compiled_resources[0].checksum,
                Checksum::from([7u8; 32])
            );
            assert_eq!(output.compiled_resources[0].size, 7);
        }
    }

    #[test]
    fn remote() {
        let target_dir = std::env::current_exe().ok().map_or_else(
            || panic!("cannot find test directory"),
            |mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            },
        );

        let registry = CompilerRegistryOptions::local_compilers(target_dir).create();

        let env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let source = ResourceTypeAndId {
            kind: text_resource::TextResource::TYPE,
            id: ResourceId::new(),
        };
        let destination = ResourcePathId::from(source).push(integer_asset::IntegerAsset::TYPE);

        let transform = Transform::new(source.kind, destination.content_type());

        let (compiler, transform) = registry.find_compiler(transform).expect("valid compiler");
        let _ = compiler.compiler_hash(transform, &env).expect("valid hash");
    }
}
