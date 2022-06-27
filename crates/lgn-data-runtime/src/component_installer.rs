use crate::{AssetRegistry, AssetRegistryError, Component};
use lgn_ecs::system::EntityCommands;

/// Trait to implement a `Component` installer
pub trait ComponentInstaller: Send + Sync {
    /// # Errors
    /// return a `AssetRegistryError` if it failed to install the component
    fn install_component(
        &self,
        _asset_registry: &AssetRegistry,
        _component: &dyn Component,
        _commands: &mut EntityCommands<'_, '_, '_>,
    ) -> Result<(), AssetRegistryError> {
        Ok(())
    }
}

/// Default Component Installer implementation (clone componnet)
#[macro_export]
macro_rules! implement_default_component_installer {
    ($installer:ident, $($concrete_type:ty),*)  => {

        #[async_trait::async_trait]
        impl lgn_data_runtime::ComponentInstaller for $installer {
            fn install_component(
                &self,
                _asset_registry: &lgn_data_runtime::AssetRegistry,
                component: &dyn lgn_data_runtime::Component,
                commands: &mut EntityCommands<'_, '_, '_>,
            ) -> Result<(), lgn_data_runtime::AssetRegistryError> {
                $(if let Some(value) = component.downcast_ref::<$concrete_type>() {
                    commands.insert(value.clone());
                })*
                Ok(())
            }
        }
    };
}
