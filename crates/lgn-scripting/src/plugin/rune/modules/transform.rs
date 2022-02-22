use rune::{runtime::Protocol, Any, ContextError, Module};

use super::{ecs::Entity, math::Vec3};

#[derive(Any)]
pub(crate) struct Transform(*mut lgn_transform::prelude::Transform);

impl Transform {
    pub(crate) fn new(entity: &Entity) -> Self {
        let mut entity = entity.get_mut();
        let transform = entity
            .get_mut::<lgn_transform::prelude::Transform>()
            .unwrap()
            .into_inner();
        Self(transform as *mut lgn_transform::prelude::Transform)
    }
}

pub(crate) fn make_transform_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_transform");

    module.ty::<Transform>()?;
    module.field_fn(
        Protocol::GET,
        "translation",
        |transform: &Transform| unsafe { Vec3::new(&mut (*transform.0).translation) },
    )?;

    Ok(module)
}
