use lgn_tracing::info;
use serde::Deserialize;

use crate::{app::AppExit, App};

/// A configuration struct for automated CI testing.
///
/// It gets used when the `lgn_ci_testing` feature is enabled to automatically
/// exit a Legion app when run through the CI. This is needed because otherwise
/// Legion apps would be stuck in the game loop and wouldn't allow the CI to progress.
#[derive(Deserialize)]
pub struct CiTestingConfig {
    /// The number of frames after which Legion should exit
    pub exit_after: Option<u32>,
}

#[allow(clippy::needless_pass_by_value)]
fn ci_testing_exit_after(
    mut current_frame: lgn_ecs::prelude::Local<'_, u32>,
    ci_testing_config: lgn_ecs::prelude::Res<'_, CiTestingConfig>,
    mut app_exit_events: lgn_ecs::event::EventWriter<'_, '_, AppExit>,
) {
    if let Some(exit_after) = ci_testing_config.exit_after {
        if *current_frame > exit_after {
            app_exit_events.send(AppExit);
            info!("Exiting after {} frames. Test successful!", exit_after);
        }
    }
    *current_frame += 1;
}

pub(crate) fn setup_app(app: &mut App) -> &mut App {
    #[cfg(not(target_arch = "wasm32"))]
    let config: CiTestingConfig = {
        let filename = std::env::var("CI_TESTING_CONFIG")
            .unwrap_or_else(|_| "ci_testing_config.ron".to_string());
        ron::from_str(
            &std::fs::read_to_string(filename)
                .expect("error reading CI testing configuration file"),
        )
        .expect("error deserializing CI testing configuration file")
    };
    #[cfg(target_arch = "wasm32")]
    let config: CiTestingConfig = {
        let config = include_str!("../../../ci_testing_config.ron");
        ron::from_str(config).expect("error deserializing CI testing configuration file")
    };

    app.insert_resource(config)
        .add_system(ci_testing_exit_after);

    app
}
