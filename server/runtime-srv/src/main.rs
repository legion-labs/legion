//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)
//!

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::str::FromStr;

use clap::Arg;
use instant::Duration;
use lgn_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_core::CorePlugin;
use lgn_data_runtime::ResourceId;
use lgn_ecs::prelude::*;
use lgn_input::InputPlugin;
use lgn_presenter_window::component::PresenterWindow;
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceExtents, RenderSurfaceId},
    Renderer, RendererPlugin,
};
use lgn_telemetry::prelude::*;
use lgn_transform::prelude::*;
use lgn_utils::HashMap;
use lgn_window::{
    WindowCloseRequested, WindowCreated, WindowDescriptor, WindowId, WindowPlugin, WindowResized,
    Windows,
};
use lgn_winit::{WinitPlugin, WinitWindows};
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    let _telemetry_guard = TelemetrySystemGuard::new(Some(Box::new(
        SimpleLogger::new().with_level(LevelFilter::Info),
    )));
    let _telemetry_thread_guard = TelemetryThreadGuard::new();
    trace_scope!();

    const ARG_NAME_CAS: &str = "cas";
    const ARG_NAME_MANIFEST: &str = "manifest";
    const ARG_NAME_ROOT: &str = "root";
    const ARG_NAME_EGUI: &str = "egui";

    let args = clap::App::new("Legion Labs runtime engine")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Server that will run with runtime data, and execute world simulation, ready to be streamed to a runtime client.")
        .arg(Arg::with_name(ARG_NAME_CAS)
            .long(ARG_NAME_CAS)
            .takes_value(true)
            .help("Path to folder containing the content storage files"))
        .arg(Arg::with_name(ARG_NAME_MANIFEST)
            .long(ARG_NAME_MANIFEST)
            .takes_value(true)
            .help("Path to the game manifest"))
        .arg(Arg::with_name(ARG_NAME_ROOT)
            .long(ARG_NAME_ROOT)
            .takes_value(true)
            .help("Root object to load, usually a world"))
        .arg(Arg::with_name(ARG_NAME_EGUI)
            .long(ARG_NAME_EGUI)
            .takes_value(false)
            .help("If supplied, starts with egui enabled"))
        .get_matches();

    let content_store_addr = args
        .value_of(ARG_NAME_CAS)
        .unwrap_or("test/sample-data/temp");

    let game_manifest = args
        .value_of(ARG_NAME_MANIFEST)
        .unwrap_or("test/sample-data/runtime/game.manifest");

    let mut assets_to_load: Vec<ResourceId> = Vec::new();

    // default root object is in sample data
    // /world/sample_1.ent
    // resource-id: 97b0740f00000000fcd3242ec9691beb
    // asset-id: aad8904500000000ab5fe63eda1e1c4f
    // checksum: 3aba5061a97aa89bb522050b16081f67

    let root_asset = args
        .value_of(ARG_NAME_ROOT)
        .unwrap_or("aad8904500000000ab5fe63eda1e1c4f");
    if let Ok(asset_id) = ResourceId::from_str(root_asset) {
        assets_to_load.push(asset_id);
    }

    let standalone = true;

    let mut app = App::new();

    app
        // Start app with 60 fps
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(TransformPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            assets_to_load,
            None,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::new(
            standalone,
            args.is_present(ARG_NAME_EGUI),
        ));

    if standalone {
        let width = 1280_f32;
        let height = 720_f32;
        app.insert_resource(WindowDescriptor {
            width,
            height,
            ..WindowDescriptor::default()
        })
        .add_plugin(WindowPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_system(on_window_created.exclusive_system())
        .add_system(on_window_resized.exclusive_system())
        .add_system(on_window_close_requested.exclusive_system())
        .insert_resource(RenderSurfaces::new());
    }

    app.run();
}

fn on_window_created(
    mut commands: Commands<'_, '_>,
    mut ev_wnd_created: EventReader<'_, '_, WindowCreated>,
    wnd_list: Res<'_, Windows>,
    winit_wnd_list: Res<'_, WinitWindows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());
        let mut render_surface = RenderSurface::new(&renderer, extents);
        render_surfaces.insert(ev.id, render_surface.id());
        let winit_wnd = winit_wnd_list.get_window(ev.id).unwrap();
        render_surface
            .register_presenter(|| PresenterWindow::from_window(&renderer, winit_wnd, extents));
        commands.spawn().insert(render_surface);
    }

    drop(wnd_list);
    drop(winit_wnd_list);
    drop(renderer);
}

fn on_window_resized(
    mut ev_wnd_resized: EventReader<'_, '_, WindowResized>,
    wnd_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    render_surfaces: Res<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_resized.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let render_surface = q_render_surfaces
                .iter_mut()
                .find(|x| x.id() == *render_surface_id);
            if let Some(mut render_surface) = render_surface {
                let wnd = wnd_list.get(ev.id).unwrap();
                render_surface.resize(
                    &renderer,
                    RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height()),
                );
            }
        }
    }

    drop(wnd_list);
    drop(renderer);
    drop(render_surfaces);
}

fn on_window_close_requested(
    mut commands: Commands<'_, '_>,
    mut ev_wnd_destroyed: EventReader<'_, '_, WindowCloseRequested>,
    query_render_surface: Query<'_, '_, (Entity, &RenderSurface)>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_destroyed.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let query_result = query_render_surface
                .iter()
                .find(|x| x.1.id() == *render_surface_id);
            if let Some(query_result) = query_result {
                commands.entity(query_result.0).despawn();
            }
        }
        render_surfaces.remove(ev.id);
    }

    drop(query_render_surface);
}

struct RenderSurfaces {
    window_id_mapper: HashMap<WindowId, RenderSurfaceId>,
}

impl RenderSurfaces {
    pub fn new() -> Self {
        Self {
            window_id_mapper: HashMap::default(),
        }
    }

    pub fn insert(&mut self, window_id: WindowId, render_surface_id: RenderSurfaceId) {
        let result = self.window_id_mapper.insert(window_id, render_surface_id);
        assert!(result.is_none());
    }

    pub fn remove(&mut self, window_id: WindowId) {
        let result = self.window_id_mapper.remove(&window_id);
        assert!(result.is_some());
    }

    pub fn get_from_window_id(&self, window_id: WindowId) -> Option<&RenderSurfaceId> {
        self.window_id_mapper.get(&window_id)
    }
}
