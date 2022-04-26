use crate::{
    error::{Error, Result},
    types::ElectronRuntimeConfiguration,
    utils::ElectronPackageConfig,
};

pub fn run(
    _electron_package_config: &ElectronPackageConfig,
    _configuration: &ElectronRuntimeConfiguration,
) -> Result<()> {
    Err(Error::UnimplementedCommand("build".into()))
}
