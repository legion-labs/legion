use rust_embed::RustEmbed;

/*
fn main() {
    legion_app::App::new()
        .insert_resource(legion_sciter::ToolWindowDescription {
            width: 0.0,
            height: 0.0,
            title: None,
            html: Some(HTML),
            url: None,
        })
        .add_plugin(legion_sciter::SciterPlugin::default())
        .run();
}

*/

use sciter_js::{sciter, window};

#[derive(RustEmbed)]
#[folder = "static/"]
#[prefix = "this://app/"]
struct StaticFolder;

fn main() {
    sciter::set_global_options(sciter::GlobalOption::ScriptRuntimeFeatures(
        sciter::ScriptRuntimeFeatures::ALLOW_FILE_IO // Enables `Sciter.machineName()`.  Required for opening file dialog (`view.selectFile()`)
         | sciter::ScriptRuntimeFeatures::ALLOW_SYSINFO, // Enables opening file dialog (`view.selectFile()`)
    ))
    .unwrap();

    sciter::set_global_options(sciter::GlobalOption::UxTheming(true)).unwrap();
    sciter::set_global_options(sciter::GlobalOption::DebugMode(true)).unwrap();

    let mut window = window::WindowBuilder::main().build();

    window.set_embedded_source::<StaticFolder>();
    window.load_file("this://app/main.htm");

    window.show();
    window::run_event_loop(|| {});
}
