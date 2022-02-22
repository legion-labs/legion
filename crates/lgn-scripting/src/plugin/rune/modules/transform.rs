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

    // fn get(&self) -> &lgn_transform::prelude::Transform {
    //     #![allow(unsafe_code)]
    //     unsafe { &*self.0 }
    // }

    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self) -> &mut lgn_transform::prelude::Transform {
        #![allow(unsafe_code)]
        unsafe { &mut *self.0 }
    }
}

pub(crate) fn make_transform_module() -> Result<Module, ContextError> {
    #![allow(unsafe_code)]
    let mut module = Module::with_crate("lgn_transform");

    module.ty::<Transform>()?;
    module.field_fn(Protocol::GET, "translation", |transform: &Transform| {
        Vec3::new(&mut transform.get_mut().translation)
    })?;

    Ok(module)
}
