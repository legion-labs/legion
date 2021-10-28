use std::{
    cell::RefCell,
    io,
    path::{Path, PathBuf},
};

use crate::{manifest::Manifest, ResourceId};
use legion_content_store::{ContentStore, ContentStoreAddr};

use super::Device;

/// Storage device that builds resources on demand. Resources are accessed through a manifest access table.
pub(crate) struct BuildDevice {
    manifest: RefCell<Manifest>,
    content_store: Box<dyn ContentStore>,
    databuild_bin: PathBuf,
    cas_addr: ContentStoreAddr,
    buildindex: PathBuf,
}

impl BuildDevice {
    pub(crate) fn new(
        manifest: Manifest,
        content_store: Box<dyn ContentStore>,
        cas_addr: ContentStoreAddr,
        build_bin: impl AsRef<Path>,
        buildindex: impl AsRef<Path>,
    ) -> Self {
        Self {
            manifest: RefCell::new(manifest),
            content_store,
            databuild_bin: build_bin.as_ref().to_owned(),
            cas_addr,
            buildindex: buildindex.as_ref().to_owned(),
        }
    }
}

impl Device for BuildDevice {
    fn lookup(&self, id: ResourceId) -> Option<Vec<u8>> {
        let output = self.build_resource(id).ok()?;
        self.manifest.borrow_mut().extend(output);

        let (checksum, size) = self.manifest.borrow().find(id)?;
        let content = self.content_store.read(checksum)?;
        assert_eq!(content.len(), size);
        Some(content)
    }
}

impl BuildDevice {
    fn build_resource(&self, resource_id: ResourceId) -> io::Result<Manifest> {
        let mut command = build_command(
            &self.databuild_bin,
            resource_id,
            &self.cas_addr,
            &self.buildindex,
        );
        let output = command.output()?;

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
    resource_id: ResourceId,
    cas: &ContentStoreAddr,
    buildindex_path: impl AsRef<Path>,
) -> std::process::Command {
    let target = "game";
    let platform = "windows";
    let locale = "en";
    let mut command = std::process::Command::new(databuild_path.as_ref());
    command.arg("compile");
    command.arg(format!("{}", resource_id));
    command.arg("--rt");
    command.arg(format!("--cas={}", cas));
    command.arg(format!("--target={}", target));
    command.arg(format!("--platform={}", platform));
    command.arg(format!("--locale={}", locale));
    command.arg(format!(
        "--buildindex={}",
        buildindex_path.as_ref().to_str().unwrap()
    ));
    command
}
