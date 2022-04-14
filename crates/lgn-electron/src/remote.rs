use crate::{
    error::Result,
    types::ElectronRuntimeConfiguration,
    utils::{ElectronPackageConfig, NpxCommand},
};

pub fn run(
    electron_package_config: &ElectronPackageConfig,
    configuration: &ElectronRuntimeConfiguration,
) -> Result<()> {
    if electron_package_config.is_typescript() {
        NpxCommand::new(electron_package_config)?.run_tsc()?;
    }

    NpxCommand::new(electron_package_config)?.run_electron(configuration)?;

    Ok(())
}
