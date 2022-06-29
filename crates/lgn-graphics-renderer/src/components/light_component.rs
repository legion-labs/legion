use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use lgn_math::Vec3;
use lgn_transform::prelude::GlobalTransform;
use lgn_utils::HashMap;

use crate::{
    core::{
        InsertRenderObjectCommand, PrimaryTableCommandBuilder, PrimaryTableView,
        RemoveRenderObjectCommand, RenderObjectId, UpdateRenderObjectCommand,
    },
    debug_display::DebugDisplay,
    lighting::RenderLight,
    picking::{PickingId, PickingIdContext, PickingManager},
    resources::DefaultMeshType,
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
    pub render_object_id: Option<RenderObjectId>,
    pub picking_id: Option<PickingId>,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::OmniDirectional,
            color: Color::WHITE,
            radiance: 40.0,
            cone_angle: 0.0,
            enabled: true,
            picking_id: None,
            render_object_id: None,
        }
    }
}

struct LightDynamicData {
    render_object_id: RenderObjectId,
    picking_id: PickingId,
}

pub(crate) struct EcsToRenderLight {
    view: PrimaryTableView<RenderLight>,
    map: HashMap<Entity, LightDynamicData>,
}

impl EcsToRenderLight {
    pub fn new(view: PrimaryTableView<RenderLight>) -> Self {
        Self {
            map: HashMap::new(),
            view,
        }
    }

    pub fn alloc_id(&self) -> RenderObjectId {
        self.view.allocate()
    }

    pub fn command_builder(&self) -> PrimaryTableCommandBuilder {
        self.view.command_builder()
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_light_components(
    mut q_changes: Query<
        '_,
        '_,
        (Entity, &GlobalTransform, &mut LightComponent),
        Or<(Changed<GlobalTransform>, Changed<LightComponent>)>,
    >,
    q_removals: RemovedComponents<'_, LightComponent>,
    mut ecs_to_render: ResMut<'_, EcsToRenderLight>,
    picking_manager: Res<'_, PickingManager>,
) {
    let mut render_commands = ecs_to_render.command_builder();

    for e in q_removals.iter() {
        let light_dynamic_data = ecs_to_render.map.remove(&e);
        if let Some(light_dynamic_data) = light_dynamic_data {
            render_commands.push(RemoveRenderObjectCommand {
                render_object_id: light_dynamic_data.render_object_id,
            });
        }
    }

    let mut picking_context = PickingIdContext::new(&picking_manager);

    for (e, transform, mut light) in q_changes.iter_mut() {
        if let Some(render_object_id) = light.render_object_id {
            // Update picking_id in hash map.
            ecs_to_render.map.insert(
                e,
                LightDynamicData {
                    render_object_id,
                    picking_id: light.picking_id.unwrap(),
                },
            );

            render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                render_object_id,
                data: (transform, light.as_ref()).into(),
            });
        } else {
            let is_already_inserted = ecs_to_render.map.contains_key(&e);
            let light_dynamic_data = if is_already_inserted {
                // This happens when the manipulator is released. The component gets recreated but we do not
                // go into the removals code above for some reason. So we need to handle it ourselves.
                ecs_to_render.map.get(&e).unwrap()
            } else {
                let render_object_id = ecs_to_render.alloc_id();
                let picking_id = picking_context.acquire_picking_id(e);

                ecs_to_render.map.insert(
                    e,
                    LightDynamicData {
                        render_object_id,
                        picking_id,
                    },
                );

                ecs_to_render.map.get(&e).unwrap()
            };

            let render_object_id = light_dynamic_data.render_object_id;
            let picking_id = light_dynamic_data.picking_id;
            light.render_object_id = Some(render_object_id);
            light.picking_id = Some(picking_id);

            if is_already_inserted {
                render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: (transform, light.as_ref()).into(),
                });
            } else {
                render_commands.push(InsertRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: (transform, light.as_ref()).into(),
                });
            }
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn tmp_debug_display_lights(
    debug_display: Res<'_, DebugDisplay>,
    lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
) {
    if lights.is_empty() {
        return;
    }

    debug_display.create_display_list(|builder| {
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
                                    - transform.rotation.mul_vec3(Vec3::new(0.0, 0.0, 0.3)), // assumes arrow length to be 0.3
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
                                transform.translation - transform.rotation.mul_vec3(Vec3::Z), // assumes cone height to be 1.0
                            )
                            .with_scale(Vec3::new(factor, factor, 1.0))
                            .with_rotation(transform.rotation),
                        DefaultMeshType::Cone,
                        Color::WHITE,
                    );
                }
                LightType::OmniDirectional => (),
            }
        }
    });
}
