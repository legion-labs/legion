use std::path::{Path, PathBuf};

use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::{manifest::Manifest, ResourceId, ResourceType, ResourceTypeAndId};

/// Builds necessary derived resources based on source resources chnaged.
pub struct BuildManager {
    build: DataBuild,
    compile_env: CompilationEnv,
    manifest: PathBuf,
}

impl BuildManager {
    /// New instance of `BuildManager`.
    pub fn new(
        options: &DataBuildOptions,
        project_dir: impl AsRef<Path>,
        manifest: impl AsRef<Path>,
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
            manifest: manifest.as_ref().to_path_buf(),
        }
    }

    /// Builds derived resources based on changed source resoure.
    pub fn build_all_derived(&mut self, resource_id: ResourceTypeAndId) -> anyhow::Result<Manifest> {
        // TODO HACK. Assume DebugCube until proper mapping is exposed
        let derived_id =
            ResourcePathId::from(resource_id).push(ResourceType::new(b"runtime_debugcube"));

        // todo: support errors
        self.build.source_pull().unwrap();
        match self.build.compile(
            derived_id,
            None, /*Some(self.manifest.clone())*/
            &self.compile_env,
        ) {
            Ok(output) => {
                println!("{:?}", self.manifest);
                let rt_manifest = output.into_rt_manifest(|_rpid| true);
                //merge into cas manifest in asset registry
                Ok(rt_manifest)
            }
            Err(e) => {
                println!("'{} {:?}'", e.to_string(), self.manifest);
                Err(anyhow::Error::new(e))
            }
        }
    }
}
