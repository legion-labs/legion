use lgn_data_runtime::Handle;
use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_transform::prelude::GlobalTransform;
use lgn_utils::HashMap;

use crate::{
    core::{PrimaryTableView, RenderObjectId},
    features::RenderVisual,
    resources::RenderModel,
};

#[derive(Component)]
pub struct VisualComponent {
    color: Color,
    color_blend: f32,
    render_model_handle: Handle<RenderModel>,
    render_object_id: Option<RenderObjectId>,
    picking_id: u32,
}

impl VisualComponent {
    pub fn new(render_model_handle: &Handle<RenderModel>, color: Color, color_blend: f32) -> Self {
        Self {
            color,
            color_blend,
            render_model_handle: render_model_handle.clone(),
            render_object_id: None,
            picking_id: 0,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_blend(&self) -> f32 {
        self.color_blend
    }

    pub fn picking_id(&self) -> u32 {
        self.picking_id
    }

    pub fn render_model_handle(&self) -> &Handle<RenderModel> {
        &self.render_model_handle
    }
}

pub(crate) struct EcsToRenderVisual {
    view: PrimaryTableView<RenderVisual>,
    map: HashMap<Entity, RenderObjectId>,
}

impl EcsToRenderVisual {
    pub fn new(view: PrimaryTableView<RenderVisual>) -> Self {
        Self {
            map: HashMap::new(),
            view,
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_visual_components(
    mut queries: ParamSet<
        '_,
        '_,
        (
            Query<
                '_,
                '_,
                (&GlobalTransform, &mut VisualComponent),
                Or<(Changed<GlobalTransform>, Changed<VisualComponent>)>,
            >,
            Query<'_, '_, (Entity, &VisualComponent), Added<VisualComponent>>,
        ),
    >,

    q_removals: RemovedComponents<'_, VisualComponent>,
    mut ecs_to_render: ResMut<'_, EcsToRenderVisual>,
) {
    // Base path. Can be simplfied more by having access to the data of removed components
    {
        let mut writer = ecs_to_render.view.writer();

        for e in q_removals.iter() {
            let render_object_id = ecs_to_render.map.get(&e);
            if let Some(render_object_id) = render_object_id {
                writer.remove(*render_object_id);
            }
        }

        for (transform, mut visual) in queries.p0().iter_mut() {
            if let Some(render_object_id) = visual.render_object_id {
                writer.update(render_object_id, (transform, visual.as_ref()).into());
            } else {
                visual.render_object_id = Some(writer.insert((transform, visual.as_ref()).into()));
            };
        }
    }
    // Update map because of removed components
    {
        let map = &mut ecs_to_render.map;

        for e in q_removals.iter() {
            map.remove(&e);
        }

        for (e, visual) in queries.p1().iter() {
            map.insert(e, visual.render_object_id.unwrap());
        }
    }
}
