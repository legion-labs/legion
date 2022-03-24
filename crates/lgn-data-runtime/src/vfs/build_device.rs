use std::time::Instant;
use std::{
    io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use lgn_content_store::{ContentStore, ContentStoreAddr};
use lgn_tracing::info;

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Storage device that builds resources on demand. Resources are accessed
/// through a manifest access table.
pub(crate) struct BuildDevice {
    manifest: Manifest,
    content_store: Box<dyn ContentStore>,
    databuild_bin: PathBuf,
    cas_addr: ContentStoreAddr,
    output_db_addr: String,
    project: PathBuf,
    force_recompile: bool,
}

impl BuildDevice {
    pub(crate) fn new(
        manifest: Manifest,
        content_store: Box<dyn ContentStore>,
        cas_addr: ContentStoreAddr,
        build_bin: impl AsRef<Path>,
        output_db_addr: String,
        project: impl AsRef<Path>,
        force_recompile: bool,
    ) -> Self {
        Self {
            manifest,
            content_store,
            databuild_bin: build_bin.as_ref().to_owned(),
            cas_addr,
            output_db_addr,
            project: project.as_ref().to_owned(),
            force_recompile,
        }
    }
}

#[async_trait]
impl Device for BuildDevice {
    async fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if self.force_recompile {
            self.reload(type_id).await
        } else {
            let (checksum, size) = self.manifest.find(type_id)?;
            let content = self.content_store.read(checksum).await?;
            assert_eq!(content.len(), size);
            Some(content)
        }
    }

    async fn reload(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        let output = self.build_resource(type_id).ok()?;
        self.manifest.extend(output);

        let (checksum, size) = self.manifest.find(type_id)?;
        let content = self.content_store.read(checksum).await?;
        assert_eq!(content.len(), size);
        Some(content)
    }
}

impl BuildDevice {
    fn build_resource(&self, resource_id: ResourceTypeAndId) -> io::Result<Manifest> {
        let mut command = build_command(
            &self.databuild_bin,
            resource_id,
            &self.cas_addr,
            &self.output_db_addr,
            &self.project,
        );

        info!("Running DataBuild for ResourceId: {}", resource_id);
        let start = Instant::now();
        let output = command.output()?;

        info!(
            "{} DataBuild for Resource: {} processed in {:?}",
            if output.status.success() {
                "Succeeded"
            } else {
                "Failed"
            },
            resource_id,
            start.elapsed(),
        );

        if !output.status.success() {
            eprintln!(
                "{:?}",
                std::str::from_utf8(&output.stdout).expect("valid utf8")
            );
            eprintln!(
                "{:?}",
                std::str::from_utf8(&output.stderr).expect("valid utf8")
            );

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Data Build Failed: '{}'",
                    std::str::from_utf8(&output.stderr).expect("valid utf8")
                ),
            ));
        }

        let manifest: Manifest = serde_json::from_slice(&output.stdout).map_err(|_e| {
            std::io::Error::new(io::ErrorKind::InvalidData, "Failed to read manifest")
        })?;

        Ok(manifest)
    }
}

fn build_command(
    databuild_path: impl AsRef<Path>,
    resource_id: ResourceTypeAndId,
    cas: &ContentStoreAddr,
    output_db_addr: &str,
    project: impl AsRef<Path>,
) -> std::process::Command {
    let target = "game";
    let platform = "windows";
    let locale = "en";
    let mut command = std::process::Command::new(databuild_path.as_ref());
    command.arg("compile");
    command.arg(resource_id.to_string());
    command.arg("--rt");
    command.arg(format!("--cas={}", cas));
    command.arg(format!("--target={}", target));
    command.arg(format!("--platform={}", platform));
    command.arg(format!("--locale={}", locale));
    command.arg(format!("--output={}", output_db_addr));
    command.arg(format!("--project={}", project.as_ref().to_str().unwrap()));
    command
}
