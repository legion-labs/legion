use std::marker::PhantomData;

use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use lgn_transform::prelude::GlobalTransform;
use lgn_utils::HashMap;

use crate::{
    core::{
        AsSpatialRenderObject, InsertRenderObjectCommand, RemoveRenderObjectCommand, RenderObject,
        RenderObjectAllocator, RenderObjectId, UpdateRenderObjectCommand,
    },
    lighting::RenderLight,
    Renderer,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    OmniDirectional,
    Directional,
    Spot,
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Color,
    pub radiance: f32,
    pub cone_angle: f32, // Spot light
    pub enabled: bool,
    pub picking_id: u32,
    pub render_object_id: Option<RenderObjectId>,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::OmniDirectional,
            color: Color::WHITE,
            radiance: 40.0,
            cone_angle: 0.0,
            enabled: true,
            picking_id: 0,
            render_object_id: None,
        }
    }
}

impl AsSpatialRenderObject<RenderLight> for LightComponent {
    fn as_spatial_render_object(&self, transform: GlobalTransform) -> RenderLight {
        RenderLight {
            transform,
            light_type: self.light_type,
            color: self.color,
            radiance: self.radiance,
            cone_angle: self.cone_angle,
            enabled: self.enabled,
            picking_id: self.picking_id,
        }
    }
}

pub(crate) struct EcsToRender<C, R> {
    map: HashMap<Entity, RenderObjectId>,
    phantom: PhantomData<C>,
    phantom2: PhantomData<R>,
}

impl<C, R> EcsToRender<C, R>
where
    R: RenderObject,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_light_components(
    renderer: Res<'_, Renderer>,
    mut q_changes: Query<
        '_,
        '_,
        (Entity, &GlobalTransform, &mut LightComponent),
        Changed<LightComponent>,
    >,
    q_removals: RemovedComponents<'_, LightComponent>,
    mut ecs_to_render: ResMut<'_, EcsToRender<LightComponent, RenderLight>>,
) {
    let mut render_commands = renderer.render_command_builder();

    for e in q_removals.iter() {
        let render_object_id = ecs_to_render.map.get(&e);
        if let Some(render_object_id) = render_object_id {
            render_commands.push(RemoveRenderObjectCommand {
                render_object_id: *render_object_id,
            });
        }
    }

    renderer.allocate_render_object(|allocator: &mut RenderObjectAllocator<'_, RenderLight>| {
        for (e, transform, mut c) in q_changes.iter_mut() {
            if let Some(render_object_id) = c.render_object_id {
                render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: c.as_spatial_render_object(*transform),
                });
            } else {
                let render_object_id = allocator.alloc();

                assert!(!ecs_to_render.map.contains_key(&e));
                ecs_to_render.map.insert(e, render_object_id);

                c.render_object_id = Some(render_object_id);
                render_commands.push(InsertRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: c.as_spatial_render_object(*transform),
                });
            };
        }
    });
}
