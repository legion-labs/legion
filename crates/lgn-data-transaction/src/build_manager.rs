use lgn_data_build::{DataBuild, DataBuildOptions, Error};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_offline::{resource::Project, ResourcePathId};
use lgn_data_runtime::{manifest::Manifest, ResourceType, ResourceTypeAndId};
use lgn_tracing::{error, info};

/// Builds necessary derived resources based on source resources changed.
pub struct BuildManager {
    build: DataBuild,
    compile_env: CompilationEnv,
    manifest: Manifest,
}

impl BuildManager {
    /// New instance of `BuildManager`.
    pub async fn new(
        options: DataBuildOptions,
        project: &Project,
        manifest: Manifest,
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
            manifest,
        })
    }

    /// Builds derived resources based on changed source resource.
    pub async fn build_all_derived(
        &mut self,
        resource_id: ResourceTypeAndId,
        project: &Project,
    ) -> Result<(ResourcePathId, Vec<ResourceTypeAndId>), Error> {
        let start = std::time::Instant::now();
        // TODO HACK. Assume DebugCube until proper mapping is exposed
        let runtime_type = if resource_id.kind == ResourceType::new(b"offline_debugcube") {
            ResourceType::new(b"runtime_debugcube")
        } else if resource_id.kind == ResourceType::new(b"offline_testentity") {
            ResourceType::new(b"runtime_testentity")
        } else if resource_id.kind == ResourceType::new(b"offline_entity") {
            ResourceType::new(b"runtime_entity")
        } else if resource_id.kind == ResourceType::new(b"offline_material") {
            ResourceType::new(b"runtime_material")
        } else if resource_id.kind == ResourceType::new(b"offline_script") {
            ResourceType::new(b"runtime_script")
        } else {
            error!(
                "Data Build {} Failed: Cannot find runtime type mapping",
                resource_id
            );
            resource_id.kind
        };

        let derived_id = ResourcePathId::from(resource_id).push(runtime_type);

        self.build.source_pull(project).await?;
        match self.build.compile_with_manifest(
            derived_id.clone(),
            &self.compile_env,
            Some(&self.manifest),
        ) {
            Ok(output) => {
                info!(
                    "Data build {} Succeeded ({:?})",
                    resource_id,
                    start.elapsed(),
                );
                let rt_manifest = output.into_rt_manifest(|_rpid| true);
                let built = rt_manifest.resources();
                Ok((derived_id, built))
            }
            Err(e) => {
                error!("Data Build {} Failed: '{}'", resource_id, e);
                Err(e)
            }
        }
    }

    /// Runtime manifest
    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }
}
