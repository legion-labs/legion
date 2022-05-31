use std::marker::PhantomData;

use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use lgn_math::Vec3;
use lgn_transform::prelude::GlobalTransform;
use lgn_utils::HashMap;

use crate::{
    core::{
        AsSpatialRenderObject, InsertRenderObjectCommand, RemoveRenderObjectCommand, RenderObject,
        RenderObjectAllocator, RenderObjectId, UpdateRenderObjectCommand,
    },
    debug_display::DebugDisplay,
    lighting::RenderLight,
    resources::DefaultMeshType,
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

    // The following fields are dynamic, meaning they do not come from serialized data.
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

struct LightDynamicData {
    render_object_id: RenderObjectId,
    picking_id: u32,
}

pub(crate) struct EcsToRender<C, R> {
    map: HashMap<Entity, LightDynamicData>,
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
        Or<(Changed<GlobalTransform>, Changed<LightComponent>)>,
    >,
    q_removals: RemovedComponents<'_, LightComponent>,
    mut ecs_to_render: ResMut<'_, EcsToRender<LightComponent, RenderLight>>,
) {
    let mut render_commands = renderer.render_command_builder();

    for e in q_removals.iter() {
        let light_dynamic_data = ecs_to_render.map.remove(&e);
        if let Some(light_dynamic_data) = light_dynamic_data {
            render_commands.push(RemoveRenderObjectCommand {
                render_object_id: light_dynamic_data.render_object_id,
            });
        }
    }

    renderer.allocate_render_object(|allocator: &mut RenderObjectAllocator<'_, RenderLight>| {
        for (e, transform, mut light) in q_changes.iter_mut() {
            if let Some(render_object_id) = light.render_object_id {
                // Update picking_id in hash map.
                ecs_to_render.map.insert(
                    e,
                    LightDynamicData {
                        render_object_id,
                        picking_id: light.picking_id,
                    },
                );

                render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: light.as_spatial_render_object(*transform),
                });
            } else {
                let is_already_inserted = ecs_to_render.map.contains_key(&e);
                let light_dynamic_data = if is_already_inserted {
                    // This happens when the manipulator is released. The component gets recreated but we do not
                    // go into the removals code above for some reason. So we need to handle it ourselves.
                    ecs_to_render.map.get(&e).unwrap()
                } else {
                    let render_object_id = allocator.alloc();

                    ecs_to_render.map.insert(
                        e,
                        LightDynamicData {
                            render_object_id,
                            picking_id: 0,
                        },
                    );

                    ecs_to_render.map.get(&e).unwrap()
                };

                let render_object_id = light_dynamic_data.render_object_id;
                light.render_object_id = Some(render_object_id);

                if is_already_inserted {
                    // Component was recreated; assign the old picking_id back to it.
                    light.picking_id = light_dynamic_data.picking_id;

                    render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                        render_object_id,
                        data: light.as_spatial_render_object(*transform),
                    });
                } else {
                    render_commands.push(InsertRenderObjectCommand::<RenderLight> {
                        render_object_id,
                        data: light.as_spatial_render_object(*transform),
                    });
                }
            };
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn tmp_debug_display_lights(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
) {
    if lights.is_empty() {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for (light, transform) in lights.iter() {
                builder.add_default_mesh(
                    &GlobalTransform::identity()
                        .with_translation(transform.translation)
                        .with_scale(Vec3::new(0.2, 0.2, 0.2)) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                        .with_rotation(transform.rotation),
                    DefaultMeshType::Sphere,
                    Color::WHITE,
                );
                match light.light_type {
                    LightType::Directional => {
                        builder.add_default_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation
                                        - transform.rotation.mul_vec3(Vec3::new(0.0, 0.3, 0.0)), // assumes arrow length to be 0.3
                                )
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Arrow,
                            Color::WHITE,
                        );
                    }
                    LightType::Spot => {
                        let factor = 4.0 * (light.cone_angle / 2.0).tan(); // assumes that default cone mesh has 1 to 4 ratio between radius and height
                        builder.add_default_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation - transform.rotation.mul_vec3(Vec3::Y), // assumes cone height to be 1.0
                                )
                                .with_scale(Vec3::new(factor, 1.0, factor))
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Cone,
                            Color::WHITE,
                        );
                    }
                    LightType::OmniDirectional => (),
                }
            }
        });
    });
}
