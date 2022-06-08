use lgn_ecs::prelude::{Res, ResMut};

use crate::{egui::Egui, RenderScope, Renderer};

#[derive(Default)]
pub(crate) struct RendererOptions {
    pub(crate) show_bounding_spheres: bool,
    pub(crate) frame_times: Vec<f32>,
    pub(crate) frame_time_average: f32,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_renderer_options(
    egui: Res<'_, Egui>,
    mut renderer_options: ResMut<'_, RendererOptions>,
    renderer: Res<'_, Renderer>,
) {
    let render_scope = renderer.render_resources().get::<RenderScope>();

    egui.window("Renderer options", |ui| {
        ui.set_min_width(400.0);

        ui.checkbox(
            &mut renderer_options.show_bounding_spheres,
            "Show bounding spheres",
        );

        let frame_time_ms = render_scope.frame_time().as_secs_f32() * 1000.0;

        renderer_options.frame_times.push(frame_time_ms);
        let num_frames_to_plot = 240;
        while renderer_options.frame_times.len() > num_frames_to_plot {
            renderer_options.frame_times.remove(0);
        }

        let values: Vec<egui::plot::Value> = renderer_options
            .frame_times
            .iter()
            .enumerate()
            .map(|(i, v)| egui::plot::Value::new(f64::from(i as u32), f64::from(*v)))
            .collect();
        let line = egui::plot::Line::new(egui::plot::Values::from_values(values))
            .color(egui::epaint::Color32::GREEN);
        egui::plot::Plot::new("Render times")
            .view_aspect(3.0)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .include_y(16.0)
            .show(ui, |plot_ui| plot_ui.line(line));

        let n = 60.0;
        renderer_options.frame_time_average =
            renderer_options.frame_time_average * (n - 1.0) / n + frame_time_ms / n;
        ui.label(&format!(
            "Renderer CPU frame time (average of last {} frames): {:.2} ms",
            n, renderer_options.frame_time_average,
        ));
    });
}
