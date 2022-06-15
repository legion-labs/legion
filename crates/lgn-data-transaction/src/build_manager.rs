use std::sync::Arc;

use lgn_content_store::indexing::{ResourceIndex, ResourceWriter, SharedTreeIdentifier};
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
        options: DataBuildOptions,
        project: &Project,
        runtime_manifest_id: SharedTreeIdentifier,
    ) -> Result<Self, Error> {
        let editor_env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let build = options.open_or_create(project).await?;

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

        let derived_id = Self::get_derived_id(resource_id);

        let data_provider = Arc::clone(self.build.get_provider());
        let indexer = new_resource_type_and_id_indexer();
        let start_manifest = ResourceIndex::new_exclusive_with_id(
            Arc::clone(&data_provider),
            indexer.clone(),
            self.runtime_manifest_id.read(),
        )
        .enumerate_resources()
        .await?;

        self.build.source_pull(project).await?;
        match self
            .build
            .compile(derived_id.clone(), &self.compile_env)
            .await
        {
            Ok(output) => {
                let runtime_manifest_id = output
                    .into_rt_manifest(Arc::clone(&data_provider), |_rpid| true)
                    .await;
                let runtime_manifest = ResourceIndex::new_exclusive_with_id(
                    Arc::clone(&data_provider),
                    indexer.clone(),
                    runtime_manifest_id,
                )
                .enumerate_resources()
                .await?;

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

                let mut runtime_manifest = ResourceIndex::new_exclusive_with_id(
                    Arc::clone(&data_provider),
                    indexer.clone(),
                    self.runtime_manifest_id.read(),
                );
                for (index_key, resource_id) in added_resources {
                    runtime_manifest
                        .add_resource(&index_key, resource_id)
                        .await?;
                }
                for (index_key, resource_id, old_resource_id) in &changed_resources {
                    let replaced_id = runtime_manifest
                        .replace_resource(index_key, resource_id.clone())
                        .await?;
                    assert_eq!(&replaced_id, old_resource_id);

                    data_provider.unwrite_resource(old_resource_id).await?;
                }
                self.runtime_manifest_id.write(runtime_manifest.id());

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
