use legion_app::prelude::*;
use legion_utils::{HashMap, Uuid};
use sciter_js::{sciter, window::run_event_loop, window::WindowBuilder};

#[derive(Default)]
pub struct SciterPlugin;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ToolWindowId(Uuid);

impl ToolWindowId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn primary() -> Self {
        Self(Uuid::from_u128(0))
    }

    pub fn is_primary(&self) -> bool {
        *self == Self::primary()
    }
}

pub struct ToolWindows {
    windows: HashMap<ToolWindowId, sciter_js::window::Window>,
}

//#[derive(Clone, Debug)]
pub struct ToolWindowDescription {
    pub width: f32,
    pub height: f32,
    pub title: Option<String>,
    pub html: Option<&'static [u8]>,
    pub url: Option<String>,
}

impl Default for ToolWindowDescription {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            title: None,
            html: None,
            url: None,
        }
    }
}

impl Plugin for SciterPlugin {
    fn build(&self, app: &mut App) {
        let mut tool_windows = ToolWindows {
            windows: HashMap::default(),
        };
        if let Some(setting) = app.world.get_resource_mut::<ToolWindowDescription>() {
            let mut primary_window = WindowBuilder::main().build();
            if let Some(html) = setting.html {
                sciter::set_global_options(sciter::GlobalOption::ScriptRuntimeFeatures(
                    sciter::ScriptRuntimeFeatures::ALLOW_FILE_IO // Enables `Sciter.machineName()`.  Required for opening file dialog (`view.selectFile()`)
                     | sciter::ScriptRuntimeFeatures::ALLOW_SYSINFO, // Enables opening file dialog (`view.selectFile()`)
                ))
                .unwrap();

                sciter::set_global_options(sciter::GlobalOption::UxTheming(true)).unwrap();

                // Enable debug mode for all windows, so that we can inspect them via Inspector.
                sciter::set_global_options(sciter::GlobalOption::DebugMode(true)).unwrap();

                primary_window.load_html(html, None);
                primary_window.show();
            }
            tool_windows
                .windows
                .insert(ToolWindowId::primary(), primary_window);
        }
        app.insert_non_send_resource(tool_windows)
            .set_runner(sciter_runner);
    }
}

pub fn sciter_runner(mut app: App) {
    run_event_loop(move || {
        app.update();
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
