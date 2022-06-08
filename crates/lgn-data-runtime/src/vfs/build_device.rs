use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use async_trait::async_trait;
use lgn_content_store::{
    indexing::{empty_tree_id, BasicIndexer, ResourceReader, TreeIdentifier, TreeLeafNode},
    Provider,
};
use lgn_tracing::info;

use super::Device;
use crate::{new_resource_type_and_id_indexer, ResourceTypeAndId, ResourceTypeAndIdIndexer};

/// Storage device that builds resources on demand. Resources are accessed
/// through a manifest access table.
pub(crate) struct BuildDevice {
    manifest_id: TreeIdentifier,
    provider: Arc<Provider>,
    indexer: ResourceTypeAndIdIndexer,
    databuild_bin: PathBuf,
    output_db_addr: String,
    project: PathBuf,
    force_recompile: bool,
}

impl BuildDevice {
    pub(crate) async fn new(
        manifest_id: Option<TreeIdentifier>,
        provider: Arc<Provider>,
        build_bin: impl AsRef<Path>,
        output_db_addr: String,
        project: impl AsRef<Path>,
        force_recompile: bool,
    ) -> Self {
        let empty_manifest_id = empty_tree_id(&provider).await.unwrap();
        Self {
            manifest_id: manifest_id.unwrap_or(empty_manifest_id),
            provider,
            indexer: new_resource_type_and_id_indexer(),
            databuild_bin: build_bin.as_ref().to_owned(),
            output_db_addr,
            project: project.as_ref().to_owned(),
            force_recompile,
        }
    }

    async fn load_internal(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if let Ok(Some(TreeLeafNode::Resource(leaf_id))) = self
            .indexer
            .get_leaf(&self.provider, &self.manifest_id, &type_id.into())
            .await
        {
            if let Ok(resource_bytes) = self.provider.read_resource_as_bytes(&leaf_id).await {
                return Some(resource_bytes);
            }
        }

        None
    }
}

#[async_trait]
impl Device for BuildDevice {
    async fn load(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if self.force_recompile {
            self.reload(type_id).await
        } else {
            self.load_internal(type_id).await
        }
    }

    async fn reload(&mut self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        let manifest_id = self.build_resource(type_id).ok()?;
        self.manifest_id = manifest_id;

        self.load_internal(type_id).await
    }
}

impl BuildDevice {
    fn build_resource(&self, resource_id: ResourceTypeAndId) -> io::Result<TreeIdentifier> {
        let mut command = build_command(
            &self.databuild_bin,
            resource_id,
            &self.output_db_addr,
            &self.project,
        );

        info!("Running DataBuild for ResourceId: {}", resource_id);
        info!("{:?}", command);
        let start = Instant::now();
        let output = command.output()?;
        info!("{:?}", output);

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

        let manifest_id = std::str::from_utf8(&output.stdout)
            .map_err(|_e| {
                std::io::Error::new(io::ErrorKind::InvalidData, "Failed to read manifest")
            })?
            .trim_end()
            .parse()
            .map_err(|_e| {
                std::io::Error::new(io::ErrorKind::InvalidData, "Failed to read manifest")
            })?;

        Ok(manifest_id)
    }
}

fn build_command(
    databuild_path: impl AsRef<Path>,
    resource_id: ResourceTypeAndId,
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
    command.arg(format!("--target={}", target));
    command.arg(format!("--platform={}", platform));
    command.arg(format!("--locale={}", locale));
    command.arg(format!("--output={}", output_db_addr));
    command.arg(format!("--project={}", project.as_ref().to_str().unwrap()));
    command
}
