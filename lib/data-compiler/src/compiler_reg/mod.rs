//! Compiler registry defines an interface between different compiler types (executable, in-process, etc).
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

/// Interface allowing to support multiple types of compilers - in-process, external executables.
pub trait CompilerStub: Send + Sync {
    /// Returns information about the compiler.
    fn info(&self) -> io::Result<CompilerInfo>;

    /// Returns the `CompilerHash` for the given compilation context.
    fn compiler_hash(&self, env: &CompilationEnv) -> io::Result<CompilerHash>;

    /// Triggers compilation of provided `compile_path` and returns the information about compilation output.
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

/// Options and flags which can be used to configure how a compiler registry is created.
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
        let infos = self.collect_info();

        CompilerRegistry {
            compilers: self.compilers,
            infos: Some(infos),
        }
    }

    /// Gathers info on all compilers.
    fn collect_info(&self) -> Vec<CompilerInfo> {
        // todo: panic if info already gathered
        let mut infos = Vec::with_capacity(self.compilers.len());
        for compiler in &self.compilers {
            let info = compiler.info().unwrap(); // todo: support failure
            infos.push(info);
        }
        infos
    }
}

/// A registry of data compilers.
#[derive(Default)]
pub struct CompilerRegistry {
    compilers: Vec<Box<dyn CompilerStub>>,
    infos: Option<Vec<CompilerInfo>>,
}

impl CompilerRegistry {
    /// Returns the compiler index and `CompilerHash` for a given transform and compilation context.
    pub fn get_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<(usize, CompilerHash)> {
        if let Some(infos) = &self.infos {
            if let Some(compiler_index) = infos.iter().position(|info| info.transform == transform)
            {
                let hash = self.compilers[compiler_index].compiler_hash(env)?;
                return Ok((compiler_index, hash));
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, ""))
    }

    /// Compile `compile_path` using compiler at `compiler_index`.
    #[allow(clippy::too_many_arguments)]
    pub fn compile(
        &self,
        compiler_index: usize,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        if compiler_index >= self.compilers.len() {
            return Err(CompilerError::InvalidTransform);
        }
        let compiler = &self.compilers[compiler_index];
        compiler.compile(
            compile_path,
            dependencies,
            derived_deps,
            cas_addr,
            project_dir,
            env,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lgn_content_store::{Checksum, ContentStoreAddr};
    use lgn_data_offline::{ResourcePathId, Transform};
    use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};

    use crate::{
        compiler_api::{CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError},
        CompiledResource, CompilerHash, Locale, Platform, Target,
    };

    use super::CompilerRegistryOptions;

    const TEST_TRANSFORM: Transform =
        Transform::new(ResourceType::new(b"input"), ResourceType::new(b"output"));
    const TEST_COMPILER: CompilerDescriptor = CompilerDescriptor {
        name: "test_name",
        build_version: "build0",
        code_version: "code0",
        data_version: "data0",
        transform: &TEST_TRANSFORM,
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
    fn in_proc() {
        let registry = CompilerRegistryOptions::default()
            .add_compiler(&TEST_COMPILER)
            .create();

        let env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let (compiler_index, hash) = registry.get_hash(TEST_TRANSFORM, &env).expect("valid hash");
        assert_eq!(hash, CompilerHash(7));

        let source = ResourceTypeAndId {
            t: ResourceType::new(b"input"),
            id: ResourceId::new(),
        };
        let cas = ContentStoreAddr::from(".");
        let proj_dir = PathBuf::from(".");
        let compile_path = ResourcePathId::from(source).push(ResourceType::new(b"output"));

        let result = registry.compile(
            compiler_index + 1,
            compile_path.clone(),
            &[],
            &[],
            cas.clone(),
            &proj_dir,
            &env,
        );

        assert!(matches!(
            result.unwrap_err(),
            CompilerError::InvalidTransform
        ));

        let output = registry
            .compile(
                compiler_index,
                compile_path.clone(),
                &[],
                &[],
                cas,
                &proj_dir,
                &env,
            )
            .expect("valid output");

        assert_eq!(output.compiled_resources.len(), 1);
        assert_eq!(output.compiled_resources[0].path, compile_path);
        assert_eq!(output.compiled_resources[0].checksum, Checksum::from(7));
        assert_eq!(output.compiled_resources[0].size, 7);
    }
}
