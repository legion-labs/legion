//! Compiler registry defines an interface between different compiler types
//! (executable, in-process, etc).

use core::fmt;
use std::{io, path::Path};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};

use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError, CompilerInfo,
    },
    compiler_cmd::list_compilers,
    CompiledResource, CompilerHash,
};

/// Interface allowing to support multiple types of compilers - in-process,
/// external executables. By returning multiple `CompilerInfo` via `info` the implementation
/// of `CompilerStub` can expose multiple compilers.
pub trait CompilerStub: Send + Sync {
    /// Returns information about the compiler.
    fn info(&self) -> io::Result<Vec<CompilerInfo>>;

    /// Returns the `CompilerHash` for the given compilation context and transform.
    fn compiler_hash(&self, transform: Transform, env: &CompilationEnv)
        -> io::Result<CompilerHash>;

    /// Triggers compilation of provided `compile_path` and returns the
    /// information about compilation output.
    #[allow(clippy::too_many_arguments)]
    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError>;
}

mod inproc_stub;
use inproc_stub::InProcessCompilerStub;

mod binary_stub;
use binary_stub::BinCompilerStub;

/// Options and flags which can be used to configure how a compiler registry is
/// created.
#[derive(Default)]
pub struct CompilerRegistryOptions {
    compilers: Vec<Box<dyn CompilerStub>>,
}

impl CompilerRegistryOptions {
    /// Creates `CompilerRegistry` based on provided compiler directory paths.
    pub fn from_dirs(dirs: &[impl AsRef<Path>]) -> Self {
        let compilers = list_compilers(dirs)
            .into_iter()
            .map(|info| {
                let compiler: Box<dyn CompilerStub> = Box::new(BinCompilerStub {
                    bin_path: info.path,
                });
                compiler
            })
            .collect::<Vec<Box<dyn CompilerStub>>>();

        Self { compilers }
    }

    /// Creates `CompilerRegistry` based on provided compiler directory path.
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        Self::from_dirs(std::slice::from_ref(&dir))
    }

    /// Register an in-process compiler.
    pub fn add_compiler(mut self, descriptor: &'static CompilerDescriptor) -> Self {
        let compiler = Box::new(InProcessCompilerStub { descriptor });
        self.compilers.push(compiler);
        self
    }

    /// Creates a new compiler registry based on specified options.
    pub fn create(self) -> CompilerRegistry {
        let (infos, indices) = self.collect_info();

        CompilerRegistry {
            compilers: self.compilers,
            infos,
            indices,
        }
    }

    /// Gathers info on all compilers.
    fn collect_info(&self) -> (Vec<CompilerInfo>, Vec<usize>) {
        let mut infos = vec![];
        let mut indices = vec![];
        for (index, compiler_stub) in self.compilers.iter().enumerate() {
            match compiler_stub.info() {
                Ok(compilers) => {
                    indices.extend(std::iter::repeat(index).take(compilers.len()));
                    infos.extend(compilers);
                }
                Err(_) => continue,
            }
        }

        (infos, indices)
    }
}

/// A registry of data compilers.
#[derive(Default)]
pub struct CompilerRegistry {
    compilers: Vec<Box<dyn CompilerStub>>,
    infos: Vec<CompilerInfo>,
    indices: Vec<usize>,
}

impl fmt::Debug for CompilerRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerRegistry")
            .field("infos", &self.infos)
            .finish()
    }
}

impl CompilerRegistry {
    /// Returns a reference to the compiler
    pub fn find_compiler(&self, transform: Transform) -> Option<(&dyn CompilerStub, Transform)> {
        if let Some(compiler_index) = self
            .infos
            .iter()
            .position(|info| info.transform == transform)
        {
            let stub_index = self.indices[compiler_index];
            return Some((self.compilers[stub_index].as_ref(), transform));
        }
        None
    }

    /// A list of compilers available in the registry.
    pub fn infos(&self) -> &Vec<CompilerInfo> {
        &self.infos
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lgn_content_store::{Checksum, ContentStoreAddr};
    use lgn_data_offline::{ResourcePathId, Transform};
    use lgn_data_runtime::{Resource, ResourceId, ResourceType, ResourceTypeAndId};

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
                    checksum: Checksum::from(7),
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

        let registry = CompilerRegistryOptions::from_dir(target_dir).create();

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
            let output = compiler
                .compile(compile_path.clone(), &[], &[], cas, &proj_dir, &env)
                .expect("valid output");

            assert_eq!(output.compiled_resources.len(), 1);
            assert_eq!(output.compiled_resources[0].path, compile_path);
            assert_eq!(output.compiled_resources[0].checksum, Checksum::from(7));
            assert_eq!(output.compiled_resources[0].size, 7);
        }
    }
}
