use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use legion_resources::{Project, ResourceId, ResourceType};

use crate::{
    compiledassetstore::CompiledAssetStore, CompiledAsset, Error, Locale, Platform, Target,
};

// Code-version, Dataformat-version
pub(crate) type CompilerId = u64;
type CodeVersion = i16;
type DataFormatVersion = i16;

pub struct CompilerInput<'a> {
    resource: ResourceId,
    _dependencies: &'a [ResourceId],
    asset_store: &'a mut dyn CompiledAssetStore,
    project: &'a Project,
}

pub struct CompilerInfo {
    handled_resources: &'static [ResourceType],
    code_id: CodeVersion,
    data_id: DataFormatVersion,
    compilerid_func: fn(
        code: &CodeVersion,
        data: &DataFormatVersion,
        target: Target,
        platform: Platform,
        locale: Locale,
    ) -> CompilerId,
    compile_func: fn(compiler_input: &mut CompilerInput<'_>) -> Result<Vec<CompiledAsset>, Error>,
}

impl CompilerInfo {
    pub fn compiler_id(&self, target: Target, platform: Platform, locale: Locale) -> CompilerId {
        (self.compilerid_func)(&self.code_id, &self.data_id, target, platform, locale)
    }

    pub fn compile(
        &self,
        resource: ResourceId,
        dependencies: &[ResourceId],
        asset_store: &mut dyn CompiledAssetStore,
        project: &Project,
    ) -> Result<Vec<CompiledAsset>, Error> {
        let mut compiler_input = CompilerInput {
            resource,
            _dependencies: dependencies,
            asset_store,
            project,
        };
        (self.compile_func)(&mut compiler_input)
    }
}

pub fn default_compilerid(
    code: &CodeVersion,
    data: &DataFormatVersion,
    _target: Target,
    _platform: Platform,
    _locale: Locale,
) -> CompilerId {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    hasher.finish()
}

pub struct CompilerRegistry {
    compiler_infos: Vec<&'static CompilerInfo>,
}

impl CompilerRegistry {
    pub(crate) fn new() -> Self {
        // todo(kstasik): a shortcut to create a compiler registry
        // compiler resource support should be cross-checked to avoid many compilers supporting the same resource type
        Self {
            compiler_infos: vec![&reverse_compiler::COMPILER_INFO],
        }
    }

    pub(crate) fn find(&self, resource_type: ResourceType) -> Option<&CompilerInfo> {
        self.compiler_infos
            .iter()
            .find(|&info| info.handled_resources.contains(&resource_type))
            .copied()
    }
}

mod reverse_compiler;
