use legion_app::prelude::*;

pub struct SciterPlugin;

pub struct MainToolWindow {
    pub main_tool_window: sciter_js::window::Window,
}

impl Plugin for SciterPlugin {
    fn build(&self, _app: &mut App) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
