use std::vec;

use legion_app::prelude::*;
use sciter_js::window;

#[derive(Default)]
pub struct SciterPlugin;

pub struct ToolWindows {
    pub primary_tool_window: sciter_js::window::Window,
    pub additional_tool_windows: Vec<sciter_js::window::Window>,
}

#[derive(Clone, Debug)]
pub struct ToolWindowDescription {
    pub width: f32,
    pub height: f32,
    pub title: Option<String>,
    pub html: Option<String>,
}

impl Default for ToolWindowDescription {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            title: None,
            html: None,
        }
    }
}

pub struct CreateWindow {
    desc: ToolWindowDescription,
}

impl Plugin for SciterPlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world
            .get_resource_or_insert_with(ToolWindowDescription::default)
            .to_owned();

        let tool_windows = ToolWindows {
            primary_tool_window: window::WindowBuilder::main().build(),
            additional_tool_windows: vec![],
        };

        app.insert_non_send_resource(tool_windows)
            .set_runner(sciter_runner);
    }
}

pub fn sciter_runner(app: App) {
    let tool_windows = app.world.get_non_send_resource::<ToolWindows>().unwrap();
    tool_windows.primary_tool_window.event_loop();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
