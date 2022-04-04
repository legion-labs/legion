//! Compiler Node groups data compilers and provides utilities required for data compilation.
//!
//! registry defines an interface between different compiler types
//! (executable, in-process, etc).

use core::fmt;
use std::sync::Arc;

mod binary_stub;
mod inproc_stub;

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
    use async_trait::async_trait;
    use lgn_content_store2::{Identifier, ProviderConfig};
    use std::{path::PathBuf, str::FromStr};

    use lgn_data_offline::{ResourcePathId, Transform};
    use lgn_data_runtime::{
        AssetRegistryOptions, Resource, ResourceId, ResourceType, ResourceTypeAndId,
    };

    use super::CompilerRegistryOptions;
    use crate::{
        compiler_api::{
            CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
            CompilerError, CompilerHash,
        },
        CompiledResource, Locale, Platform, Target,
    };

    const TEST_TRANSFORM: Transform =
        Transform::new(ResourceType::new(b"input"), ResourceType::new(b"output"));
    const TEST_COMPILER: CompilerDescriptor = CompilerDescriptor {
        name: "test_name",
        build_version: "build0",
        code_version: "code0",
        data_version: "data0",
        transform: &TEST_TRANSFORM,
        compiler_creator: || Box::new(TestCompiler {}),
    };

    struct TestCompiler();

    #[async_trait]
    impl Compiler for TestCompiler {
        async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
            registry
        }

        async fn hash(
            &self,
            _code: &'static str,
            _data: &'static str,
            _env: &CompilationEnv,
        ) -> CompilerHash {
            CompilerHash(7)
        }

        async fn compile(
            &self,
            ctx: CompilerContext<'_>,
        ) -> Result<CompilationOutput, CompilerError> {
            Ok(CompilationOutput {
                compiled_resources: vec![CompiledResource {
                    path: ctx.target_unnamed,
                    content_id: Identifier::new(b"AAAAAAA"),
                }],
                resource_references: vec![],
            })
        }
    }

    #[tokio::test]
    async fn binary() {
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

        let registry = CompilerRegistryOptions::local_compilers(target_dir)
            .create()
            .await;

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
        let _ = compiler
            .compiler_hash(transform, &env)
            .await
            .expect("valid hash");
    }

    #[tokio::test]
    async fn in_proc() {
        let registry = CompilerRegistryOptions::default()
            .add_compiler(&TEST_COMPILER)
            .create()
            .await;

        let env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let (compiler, transform) = registry.find_compiler(TEST_TRANSFORM).expect("a compiler");
        let hash = compiler
            .compiler_hash(transform, &env)
            .await
            .expect("valid hash");
        assert_eq!(hash, CompilerHash(7));

        let source = ResourceTypeAndId {
            kind: ResourceType::new(b"input"),
            id: ResourceId::new(),
        };
        let proj_dir = PathBuf::from(".");
        let compile_path = ResourcePathId::from(source).push(ResourceType::new(b"output"));

        let data_content_provider = ProviderConfig::default().instantiate().await.unwrap();

        // testing successful compilation
        {
            let registry = AssetRegistryOptions::new().create().await;
            let output = compiler
                .compile(
                    compile_path.clone(),
                    &[],
                    &[],
                    registry,
                    &data_content_provider,
                    &proj_dir,
                    &env,
                )
                .await
                .expect("valid output");

            assert_eq!(output.compiled_resources.len(), 1);
            assert_eq!(output.compiled_resources[0].path, compile_path);
            assert_eq!(
                output.compiled_resources[0].content_id,
                Identifier::from_str("AAAAAAA").unwrap()
            );
            assert_eq!(output.compiled_resources[0].content_id.data_size(), 7);
        }
    }
}
