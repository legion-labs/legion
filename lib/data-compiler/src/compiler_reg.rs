//! Compiler registry defines an interface between different compiler types (executable, in-process, etc).
use std::{
    io,
    path::{Path, PathBuf},
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};

use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError, CompilerInfo,
    },
    compiler_cmd::{list_compilers, CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd},
    CompiledResource, CompilerHash,
};

/// Interface allowing to support multiple types of compilers - in-process, external executables, shared objects.
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

//

struct InProcessCompilerStub {
    descriptor: &'static CompilerDescriptor,
}

impl CompilerStub for InProcessCompilerStub {
    fn compiler_hash(&self, env: &CompilationEnv) -> io::Result<CompilerHash> {
        // todo: code and data versioning
        let hash = (self.descriptor.compiler_hash_func)("todo: code", "todo: data", env);
        Ok(hash)
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        self.descriptor.compile(
            compile_path,
            dependencies,
            derived_deps,
            cas_addr,
            project_dir,
            env,
        )
    }

    fn info(&self) -> io::Result<CompilerInfo> {
        let info = CompilerInfo {
            build_version: self.descriptor.build_version.to_string(),
            code_version: self.descriptor.code_version.to_string(),
            data_version: self.descriptor.data_version.to_string(),
            transform: *self.descriptor.transform,
        };
        Ok(info)
    }
}

//

struct BinCompilerStub {
    bin_path: PathBuf,
}

impl CompilerStub for BinCompilerStub {
    fn compiler_hash(&self, env: &CompilationEnv) -> io::Result<CompilerHash> {
        let cmd = CompilerHashCmd::new(env);
        cmd.execute(&self.bin_path)
            .map(|output| output.compiler_hash)
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        resource_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        let mut cmd = CompilerCompileCmd::new(
            &compile_path,
            dependencies,
            derived_deps,
            &cas_addr,
            resource_dir,
            env,
        );

        cmd.execute(&self.bin_path)
            .map(|output| CompilationOutput {
                compiled_resources: output.compiled_resources,
                resource_references: output.resource_references,
            })
            .map_err(|_e| CompilerError::StdoutError)
    }

    fn info(&self) -> io::Result<CompilerInfo> {
        let cmd = CompilerInfoCmd::default();
        cmd.execute(&self.bin_path).map(|output| CompilerInfo {
            build_version: output.build_version,
            code_version: output.code_version,
            data_version: output.data_version,
            transform: output.transform,
        })
    }
}

/// Options and flags which can be used to configure how a compiler registry is created.
pub struct CompilerRegistryOptions {
    compilers: Vec<Box<dyn CompilerStub>>,
}

impl CompilerRegistryOptions {
    /// Creates `CompilerRegistry` based on provided compiler directory paths.
    pub fn from_dir(dirs: &[impl AsRef<Path>]) -> Self {
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

    /// Creates a new compiler registry based on specified options.
    pub fn create(mut self) -> CompilerRegistry {
        let infos = self.collect_info();

        CompilerRegistry {
            compilers: self.compilers,
            infos: Some(infos),
        }
    }

    /// Gathers info on all compilers.
    fn collect_info(&mut self) -> Vec<CompilerInfo> {
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
