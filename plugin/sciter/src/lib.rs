use legion_app::prelude::*;

pub struct SciterPlugin;

pub struct ToolWindows {
    pub primary_tool_window: sciter_js::window::Window,
    pub additional_tool_windows: Vec<sciter_js::window::Window>,
}

pub struct WindowDescriptor {
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub html: String,
}

pub struct CreateWindow {}

impl Plugin for SciterPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(sciter_runner);
    }
}

pub fn sciter_runner(_app: App) {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
