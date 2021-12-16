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
        options: &DataBuildOptions,
        project_dir: impl AsRef<Path>,
        manifest: Manifest,
    ) -> Self {
        let editor_env = CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        };

        let build = options.open_or_create(project_dir).unwrap(); // todo: failure
        Self {
            build,
            compile_env: editor_env,
            manifest,
        }
    }

    /// Builds derived resources based on changed source resoure.
    pub fn build_all_derived(&mut self, resource_id: ResourceTypeAndId) -> anyhow::Result<()> {
        // TODO HACK. Assume DebugCube until proper mapping is exposed
        let derived_id =
            ResourcePathId::from(resource_id).push(ResourceType::new(b"runtime_debugcube"));

        self.build.source_pull().unwrap();
        match self.build.compile(
            derived_id,
            None, /*Some(self.manifest.clone())*/
            &self.compile_env,
        ) {
            Ok(output) => {
                let rt_manifest = output.into_rt_manifest(|_rpid| true);
                self.manifest.extend(rt_manifest);
                Ok(())
            }
            Err(e) => {
                println!("Data Build Failed: '{}'", e.to_string());
                Err(anyhow::Error::new(e))
            }
        }
    }
}
