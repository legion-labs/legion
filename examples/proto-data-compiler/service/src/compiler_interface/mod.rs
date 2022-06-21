use futures::future::*;
use std::sync::Arc;
use strum_macros::Display;
use thiserror::Error;

use async_trait::async_trait;

use crate::build_db::BuildDb;
use crate::content_store::ContentStore;
use crate::data_execution_provider::DataExecutionProvider;
use crate::resource_manager::ResourceManager;
use crate::source_control::SourceControl;
use crate::{content_store::ContentAddr, ResourcePathId};

use crate::CompilationInputs;

#[derive(Debug, Clone)]
pub struct CompilationContent {
    pub id: ResourcePathId,
    pub addr: ContentAddr,
    pub references: Vec<ResourcePathId>,
}

#[derive(Default, Debug, Clone)]
pub struct CompilationOutput {
    pub content: Vec<CompilationContent>,
}

impl CompilationOutput {
    fn single(id: ResourcePathId, addr: ContentAddr, references: Vec<ResourcePathId>) -> Self {
        Self {
            content: vec![CompilationContent {
                id,
                addr,
                references,
            }],
        }
    }
}

pub const TEXTURE_A_CONTENT: &str = "texture a";
pub const TEXTURE_B_CONTENT: &str = "texture b";
pub const TEXTURE_C_CONTENT: &str = "texture c";
pub const MATERIAL_CONTENT: &str = "material";

pub const ATLAS_COMPILER: &str = "AtlasCompiler";
pub const ENTITY_COMPILER: &str = "EntityCompiler";
pub const MATERIAL_COMPILER: &str = "MaterialCompiler";
pub const TEST_COMPILER: &str = "TestCompiler";
pub const TEST_COMPILATION_APPEND: &str = " compiled";
pub const TEXTURE_COMPILER: &str = "TextureCompiler";

#[derive(PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd, Debug, Display)]
pub enum ResourceGuid {
    Origin,
    TextureAtlas,
    Car,
    Navmesh,
    TextureA,
    TextureB,
    TextureC,
    MeshComponent,
    CollisionComponent,
    PS5Mesh,
    CollisionMesh,
    SourceMesh,
    Material,
    ResourceA,
    ResourceB,
    ResourceC,
    ResourceD,
    ResourceE,
    ResourceF,
    ResourceG,
    ResourceH,
    ResourceI,
    ResourceJ,
    ResourceK,
    ResourceL,
    ResourceM,
    ResourceN,
    ResourceO,
    ResourceP,
    Invalid,
}

pub type CompilerType = String;

#[async_trait]
pub trait Compiler: Send + Sync {
    async fn compile(
        &self,
        compilation_inputs: CompilationInputs,
        context: &mut CompilerContext,
    ) -> Result<(), CompilerError>;

    fn get_compiler_type(&self) -> CompilerType;
}

#[derive(Debug, Clone)]
pub struct CompilerContext {
    source: ResourcePathId,
    content_store: Arc<ContentStore>,
    resource_manager: ResourceManager,
    pub loaded_resources: Vec<ResourcePathId>,
    pub output: CompilationOutput,
}

impl CompilerContext {
    pub fn new(
        source: ResourcePathId,
        content_store: Arc<ContentStore>,
        resource_manager: ResourceManager,
    ) -> Self {
        Self {
            source,
            content_store,
            resource_manager,
            loaded_resources: vec![],
            output: CompilationOutput::default(),
        }
    }

    pub async fn store(&mut self, id: ResourcePathId, content: String) -> ContentAddr {
        let addr = self.content_store.store(content).await;
        self.output.content.push(CompilationContent {
            id,
            addr,
            references: vec![],
        });
        addr
    }

    pub fn add_runtime_references(&mut self, id: ResourcePathId, refs: &[ResourcePathId]) {
        if let Some(content) = self.output.content.iter_mut().find(|a| a.id == id) {
            content.references.extend(refs.iter().cloned());
        }
    }

    pub async fn load(&mut self, id: ResourcePathId) -> Result<String, CompilerError> {
        let content = self.resource_manager.load(id.clone()).await?;
        self.loaded_resources.push(id);
        Ok(content)
    }

    pub async fn load_many(
        &mut self,
        ids: &[ResourcePathId],
    ) -> Result<Vec<String>, CompilerError> {
        let all_futures: Vec<_> = ids
            .iter()
            .cloned()
            .map(|id| {
                let resource_manager = self.resource_manager.clone();
                tokio::task::spawn(async move { resource_manager.load(id.clone()).await })
            })
            .collect::<Vec<_>>();

        let all_textures = try_join_all(all_futures)
            .await
            .map_err(|_e| CompilerError::Cancelled)?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        self.loaded_resources.extend_from_slice(ids);

        Ok(all_textures)
    }
}

/// Build parameters uniformly applied to all 'compilation units' accross a compilation request.
#[derive(Default, Debug, Clone)]
pub struct BuildParams {
    target: String,   // Client / Server
    platform: String, // Windows / Linux
    locale: String,   // en / fr
    data_build_version: String,
    feature_flags: String, // Demo, Trail, AllLevels
    code_version: i32, // code version the Runtime was compiled with - helpful to deduce compiler versions to use.
}

#[derive(Error, Debug)]
pub enum CompilerError {
    /// Not found.
    #[error("Not found")]
    NotFound,

    #[error("Wrong compiler invoked for '{0}'.")]
    WrongCompiler(String),

    #[error("Compiler not found for '{0}'.")]
    CompilerNotFound(CompilerType),

    #[error("Cancelled")]
    Cancelled,

    #[error("Invalid Argument")]
    InvalidArg,
}

pub struct Services {
    pub content_store: Arc<ContentStore>,
    pub source_control: Arc<SourceControl>,
    pub build_db: Arc<BuildDb>,
    pub data_execution_provider: Arc<dyn DataExecutionProvider>,
    pub tokio_runtime: tokio::runtime::Handle,
}
