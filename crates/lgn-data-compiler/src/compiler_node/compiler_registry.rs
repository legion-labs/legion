use std::{fmt, io, path::Path, sync::Arc};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};

use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError, CompilerInfo,
    },
    compiler_cmd::list_compilers,
    CompiledResource, CompilerHash,
};

use super::{binary_stub::BinCompilerStub, inproc_stub::InProcessCompilerStub};

/// Interface allowing to support multiple types of compilers - in-process,
/// external executables. By returning multiple `CompilerInfo` via `info` the implementation
/// of `CompilerStub` can expose multiple compilers.
pub trait CompilerStub: Send + Sync {
    /// Returns information about the compiler.
    fn info(&self) -> io::Result<Vec<CompilerInfo>>;

    /// Returns the `CompilerHash` for the given compilation context and transform.
    fn compiler_hash(&self, transform: Transform, env: &CompilationEnv)
        -> io::Result<CompilerHash>;

    /// Allow the compiler to register its own type loaders.
    fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions;

    /// Triggers compilation of provided `compile_path` and returns the
    /// information about compilation output.
    #[allow(clippy::too_many_arguments)]
    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        registry: Arc<AssetRegistry>,
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError>;
}

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

    /// Initializes all compilers allowing them to register type loaders with `AssetRegistry`.
    pub fn init_all(&self, mut registry_options: AssetRegistryOptions) -> AssetRegistryOptions {
        for compiler in &self.compilers {
            registry_options = compiler.init(registry_options);
        }
        registry_options
    }
}
