use rune::{runtime::Protocol, Any, ContextError, Module};

use super::math::Vec2;
use crate::ScriptingEventCache;

#[derive(Any)]
pub(crate) struct Events(*const ScriptingEventCache);

impl Events {
    pub(crate) fn new(event_cache: *const ScriptingEventCache) -> Self {
        Self(event_cache)
    }

    fn get(&self) -> &ScriptingEventCache {
        #![allow(unsafe_code)]
        unsafe { &*self.0 }
    }
}

pub(crate) fn make_scripting_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_scripting");

    module.ty::<Events>()?;
    module.field_fn(Protocol::GET, "mouse_motion", |events: &Events| {
        Vec2::new(&events.get().mouse_motion.delta)
    })?;

    Ok(module)
}
