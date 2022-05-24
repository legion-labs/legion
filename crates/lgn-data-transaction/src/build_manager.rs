use lgn_content_store::indexing::{
    self, BasicIndexer, ResourceWriter, SharedTreeIdentifier, TreeLeafNode,
};
use lgn_data_build::{DataBuild, DataBuildOptions, Error};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_offline::resource::Project;
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, ResourcePathId, ResourceType, ResourceTypeAndId,
};
use lgn_tracing::{error, info};

/// Builds necessary derived resources based on source resources changed.
pub struct BuildManager {
    build: DataBuild,
    compile_env: CompilationEnv,
    runtime_manifest_id: SharedTreeIdentifier,
}

impl BuildManager {
    /// Return the derived `ResourcePathId` from a `ResourceId`
    pub fn get_derived_id(resource_id: ResourceTypeAndId) -> ResourcePathId {
        // TODO HACK.
        let runtime_type = if resource_id.kind == ResourceType::new(b"offline_testentity") {
            ResourceType::new(b"runtime_testentity")
        } else if resource_id.kind == ResourceType::new(b"offline_entity") {
            ResourceType::new(b"runtime_entity")
        } else if resource_id.kind == ResourceType::new(b"offline_material") {
            ResourceType::new(b"runtime_material")
        } else if resource_id.kind == ResourceType::new(b"offline_script") {
            ResourceType::new(b"runtime_script")
        } else {
            error!(
                "Data Build {:?} Failed: Cannot find runtime type mapping",
                resource_id
            );
            resource_id.kind
        };

        ResourcePathId::from(resource_id).push(runtime_type)
    }

    /// New instance of `BuildManager`.
    pub async fn new(
        project: &Project,
        options: DataBuildOptions,
        project: &Project,
        runtime_manifest_id: SharedTreeIdentifier,
    ) -> Result<Self, Error> {
        let editor_env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let build = options.open_or_create().await?;

        Ok(Self {
            build,
            compile_env: editor_env,
            runtime_manifest_id,
        })
    }

    /// Builds derived resources based on changed source resource.
    pub async fn build_all_derived(
        &mut self,
        resource_id: ResourceTypeAndId,
        project: &Project,
    ) -> Result<(ResourcePathId, Vec<ResourceTypeAndId>), Error> {
        let start = std::time::Instant::now();

        let provider = project.get_provider();
        let derived_id = Self::get_derived_id(resource_id);

        let indexer = new_resource_type_and_id_indexer();
        let start_manifest = indexing::enumerate_resources(
            self.build.get_provider(),
            &indexer,
            &self.runtime_manifest_id.read(),
        )
        .await
        .map_err(Error::InvalidContentStoreIndexing)?;

        self.build.source_pull(project).await?;
        match self
            .build
            .compile(derived_id.clone(), &self.compile_env)
            .await
        {
            Ok(output) => {
                let data_provider = self.build.get_provider();
                let runtime_manifest_id =
                    output.into_rt_manifest(data_provider, |_rpid| true).await;
                let runtime_manifest =
                    indexing::enumerate_resources(data_provider, &indexer, &runtime_manifest_id)
                        .await
                        .map_err(Error::InvalidContentStoreIndexing)?;

                let mut added_resources = Vec::new();
                let mut changed_resources = Vec::new();
                for (index_key, resource_id) in runtime_manifest {
                    if let Some((_index_key, old_resource_id)) =
                        start_manifest.iter().find(|(key, _id)| key == &index_key)
                    {
                        if &resource_id != old_resource_id {
                            changed_resources.push((
                                index_key,
                                resource_id,
                                old_resource_id.clone(),
                            ));
                        }
                    } else {
                        added_resources.push((index_key, resource_id));
                    }
                }

                info!(
                    "Data build {:?} succeeded, {} changed assets ({:?}) ",
                    resource_id,
                    changed_resources.len(),
                    start.elapsed(),
                );

                let mut runtime_manifest_id = self.runtime_manifest_id.read();
                for (index_key, resource_id) in added_resources {
                    runtime_manifest_id = indexer
                        .add_leaf(
                            data_provider,
                            &runtime_manifest_id,
                            &index_key,
                            TreeLeafNode::Resource(resource_id),
                        )
                        .await?;
                }
                for (index_key, resource_id, old_resource_id) in &changed_resources {
                    let (manifest_id, old_node) = indexer
                        .replace_leaf(
                            data_provider,
                            &runtime_manifest_id,
                            index_key,
                            TreeLeafNode::Resource(resource_id.clone()),
                        )
                        .await?;
                    runtime_manifest_id = manifest_id;

                    if let TreeLeafNode::Resource(id) = old_node {
                        assert_eq!(&id, old_resource_id);
                    }

                    data_provider.unwrite_resource(old_resource_id).await?;
                }
                self.runtime_manifest_id.write(runtime_manifest_id);

                let changed_resources = changed_resources
                    .into_iter()
                    .map(|(index_key, _new_id, _old_id)| index_key.into())
                    .collect();

                Ok((derived_id, changed_resources))
            }
            Err(e) => {
                error!("Data Build {:?} Failed: '{}'", resource_id, e);
                Err(e)
            }
        }
    }

    /// Runtime manifest identifier
    pub fn get_runtime_manifest_id(&self) -> SharedTreeIdentifier {
        self.runtime_manifest_id.clone()
    }

    /// Return the Offline source from a runtime id
    pub async fn resolve_offline_id(
        &self,
        runtime_id: ResourceTypeAndId,
    ) -> Option<ResourceTypeAndId> {
        self.build
            .lookup_pathid(runtime_id)
            .await
            .unwrap()
            .map(|path| path.source_resource())
    }
}
