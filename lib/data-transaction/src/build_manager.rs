use std::path::Path;

use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::{manifest::Manifest, ResourceType, ResourceTypeAndId};

/// Builds necessary derived resources based on source resources chnaged.
pub struct BuildManager {
    build: DataBuild,
    compile_env: CompilationEnv,
    manifest: Manifest,
}

impl BuildManager {
    /// New instance of `BuildManager`.
    pub fn new(
        options: DataBuildOptions,
        project_dir: impl AsRef<Path>,
        manifest: Manifest,
    ) -> anyhow::Result<Self> {
        let editor_env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let build = options.open_or_create(project_dir)?;
        Ok(Self {
            build,
            compile_env: editor_env,
            manifest,
        })
    }

    /// Builds derived resources based on changed source resoure.
    pub fn build_all_derived(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> anyhow::Result<Vec<ResourceTypeAndId>> {
        let start = std::time::Instant::now();
        // TODO HACK. Assume DebugCube until proper mapping is exposed
        let runtime_type = if resource_id.t == ResourceType::new(b"offline_debugcube") {
            ResourceType::new(b"runtime_debugcube")
        } else if resource_id.t == ResourceType::new(b"offline_testentity") {
            ResourceType::new(b"runtime_testentity")
        } else {
            resource_id.t
        };

        let derived_id = ResourcePathId::from(resource_id).push(runtime_type);

        self.build.source_pull()?;
        match self.build.compile(derived_id, None, &self.compile_env) {
            Ok(output) => {
                println!(
                    "Data build {} Succeeded ({:?})",
                    resource_id,
                    start.elapsed(),
                );
                let rt_manifest = output.into_rt_manifest(|_rpid| true);
                let built = rt_manifest.resources();
                self.manifest.extend(rt_manifest);
                Ok(built)
            }
            Err(e) => {
                println!("Data Build {} Failed: '{}'", resource_id, e.to_string());
                Err(anyhow::Error::new(e))
            }
        }
    }
}
