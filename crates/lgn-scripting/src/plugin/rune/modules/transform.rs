use rune::{runtime::Protocol, Any, ContextError, Module};

use super::{ecs::Entity, math::Vec3};

#[derive(Any)]
pub(crate) struct Transform(*mut lgn_transform::prelude::Transform);

impl Transform {
    pub(crate) fn new(entity: &Entity) -> Option<Self> {
        let mut entity = entity.get_mut();
        entity
            .get_mut::<lgn_transform::prelude::Transform>()
            .map(|t| Self(t.into_inner()))
    }

    // fn get(&self) -> &mut lgn_transform::prelude::Transform {
    //     #![allow(unsafe_code)]
    //     unsafe { &*self.0 }
    // }

    // fn get_mut(&mut self) -> &mut lgn_transform::prelude::Transform {
    //     #![allow(unsafe_code)]
    //     unsafe { &mut *self.0 }
    // }
}

pub(crate) fn make_transform_module() -> Result<Module, ContextError> {
    #![allow(unsafe_code)]
    let mut module = Module::with_crate("lgn_transform");

    module.ty::<Transform>()?;
    module.field_fn(
        Protocol::GET,
        "translation",
        |transform: &Transform| unsafe { Vec3::new(&mut (*transform.0).translation) },
    )?;

    Ok(module)
}
