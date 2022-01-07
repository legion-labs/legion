use std::time::Instant;
use std::{
    cell::RefCell,
    io,
    path::{Path, PathBuf},
};

use lgn_content_store::{ContentStore, ContentStoreAddr};
use lgn_telemetry::info;

use super::Device;
use crate::{manifest::Manifest, ResourceTypeAndId};

/// Storage device that builds resources on demand. Resources are accessed
/// through a manifest access table.
pub(crate) struct BuildDevice {
    manifest: RefCell<Manifest>,
    content_store: Box<dyn ContentStore>,
    databuild_bin: PathBuf,
    cas_addr: ContentStoreAddr,
    buildindex: PathBuf,
    force_recompile: bool,
}

impl BuildDevice {
    pub(crate) fn new(
        manifest: Manifest,
        content_store: Box<dyn ContentStore>,
        cas_addr: ContentStoreAddr,
        build_bin: impl AsRef<Path>,
        buildindex: impl AsRef<Path>,
        force_recompile: bool,
    ) -> Self {
        Self {
            manifest: RefCell::new(manifest),
            content_store,
            databuild_bin: build_bin.as_ref().to_owned(),
            cas_addr,
            buildindex: buildindex.as_ref().to_owned(),
            force_recompile,
        }
    }
}

impl Device for BuildDevice {
    fn load(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        if self.force_recompile {
            self.reload(type_id)
        } else {
            let (checksum, size) = self.manifest.borrow().find(type_id)?;
            let content = self.content_store.read(checksum)?;
            assert_eq!(content.len(), size);
            Some(content)
        }
    }

    fn reload(&self, type_id: ResourceTypeAndId) -> Option<Vec<u8>> {
        let output = self.build_resource(type_id).ok()?;
        self.manifest.borrow_mut().extend(output);

        let (checksum, size) = self.manifest.borrow().find(type_id)?;
        let content = self.content_store.read(checksum)?;
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
            &self.buildindex,
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
    buildindex_dir: impl AsRef<Path>,
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
    command.arg(format!(
        "--buildindex={}",
        buildindex_dir.as_ref().to_str().unwrap()
    ));
    command
}
