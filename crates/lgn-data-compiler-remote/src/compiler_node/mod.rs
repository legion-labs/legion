//! Compiler Node groups data compilers and provides utilities required for data compilation.
//!
//! registry defines an interface between different compiler types
//! (executable, in-process, etc).

pub(crate) mod remote_data_executor;
mod remote_stub;

use std::path::Path;

use lgn_data_compiler::compiler_node::CompilerRegistryOptions;

use self::remote_stub::RemoteCompilerStub;

/// Register an uber compiler for remote compilation.
pub fn remote_compilers(
    compilers_dir: impl AsRef<Path>,
    endpoint: &str,
) -> CompilerRegistryOptions {
    CompilerRegistryOptions::from_external_compilers(std::slice::from_ref(&compilers_dir), |info| {
        Box::new(RemoteCompilerStub {
            server_addr: endpoint.to_string(),
            bin_path: info.path,
        })
    })
}

#[cfg(test)]
mod tests {
    use lgn_data_offline::{ResourcePathId, Transform};
    use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};

    use super::CompilerRegistryOptions;
    use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};

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
