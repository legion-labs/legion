//! Minimalistic Sciter sample.

static HTML: &[u8] = include_bytes!("../resources/main_window.htm");

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

/*
use sciter_js::{sciter, window};

fn main() {
    // Step 1: Include the 'minimal.html' file as a byte array.
    // Hint: Take a look into 'minimal.html' which contains some tiscript code.
    let html = include_bytes!("../resources/main_window.htm");

    // Step 2: Enable the features we need in our tiscript code.
    sciter::set_global_options(sciter::GlobalOption::ScriptRuntimeFeatures(
        sciter::ScriptRuntimeFeatures::ALLOW_FILE_IO // Enables `Sciter.machineName()`.  Required for opening file dialog (`view.selectFile()`)
         | sciter::ScriptRuntimeFeatures::ALLOW_SYSINFO, // Enables opening file dialog (`view.selectFile()`)
    ))
    .unwrap();

    sciter::set_global_options(sciter::GlobalOption::UxTheming(true)).unwrap();

    // Enable debug mode for all windows, so that we can inspect them via Inspector.
    sciter::set_global_options(sciter::GlobalOption::DebugMode(true)).unwrap();

    // Step 3: Create a new main sciter window of type `sciter::Window`.
    // Hint: The sciter Window wrapper (src/window.rs) contains more
    // interesting functions to open or attach to another existing window.
    let mut window = window::WindowBuilder::main().build();

    // Step 4: Load HTML byte array from memory to `sciter::Window`.
    // Hint: second parameter is an optional uri, it can be `None` in simple cases,
    // but it is useful for debugging purposes (check the Inspector tool from the Sciter SDK).
    // Also you can use a `load_file` method, but it requires an absolute path
    // of the main document to resolve HTML resources properly.
    window.load_html(html, Some("example://details-summary.htm"));

    // Step 5: Show window and run the main app message loop until window been closed.
    window.event_loop();
}
*/
