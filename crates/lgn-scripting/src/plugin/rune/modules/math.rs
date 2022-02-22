use std::fmt::{self, Write};

use rune::{runtime::Protocol, Any, ContextError, Module};

#[derive(Any)]
pub(crate) struct Vec2(*const lgn_math::Vec2);

impl Vec2 {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) fn new(vec: &lgn_math::Vec2) -> Self {
        Self(vec as *const lgn_math::Vec2)
    }

    fn get(&self) -> &lgn_math::Vec2 {
        unsafe { &*self.0 }
    }

    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.get())
    }
}

fn normalize2(x: f32, y: f32) -> (f32, f32) {
    let vec = lgn_math::Vec2::new(x, y);
    let vec = vec.normalize_or_zero();
    (vec.x, vec.y)
}

#[derive(Any)]
pub(crate) struct Vec3(*mut lgn_math::Vec3);

impl Vec3 {
    pub(crate) fn new(vec: &mut lgn_math::Vec3) -> Self {
        Self(vec as *mut lgn_math::Vec3)
    }

    fn get(&self) -> &lgn_math::Vec3 {
        unsafe { &*self.0 }
    }

    fn get_mut(&mut self) -> &mut lgn_math::Vec3 {
        unsafe { &mut *self.0 }
    }

    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.get())
    }

    fn clamp_x(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.x = v.x.clamp(min, max);
    }

    fn clamp_y(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.y = v.y.clamp(min, max);
    }

    fn clamp_z(&mut self, min: f32, max: f32) {
        let v = self.get_mut();
        v.z = v.z.clamp(min, max);
    }
}

fn random() -> f32 {
    rand::random::<f32>()
}

pub(crate) fn make_math_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_math");

    module.function(&["random"], random)?;
    module.function(&["normalize2"], normalize2)?;

    module.ty::<Vec2>()?;
    module.inst_fn(Protocol::STRING_DISPLAY, Vec2::display)?;
    module.field_fn(Protocol::GET, "x", |v: &Vec2| v.get().x)?;
    module.field_fn(Protocol::GET, "y", |v: &Vec2| v.get().y)?;

    module.ty::<Vec3>()?;
    module.inst_fn(Protocol::STRING_DISPLAY, Vec3::display)?;
    module.inst_fn("clamp_x", Vec3::clamp_x)?;
    module.inst_fn("clamp_y", Vec3::clamp_y)?;
    module.inst_fn("clamp_z", Vec3::clamp_z)?;
    module.field_fn(Protocol::GET, "x", |v: &Vec3| v.get().x)?;
    module.field_fn(Protocol::SET, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x = x;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x += x;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x -= x;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x *= x;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "x", |v: &mut Vec3, x: f32| {
        v.get_mut().x /= x;
    })?;
    module.field_fn(Protocol::GET, "y", |v: &Vec3| v.get().y)?;
    module.field_fn(Protocol::SET, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y = y;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y += y;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y -= y;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y *= y;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "y", |v: &mut Vec3, y: f32| {
        v.get_mut().y /= y;
    })?;
    module.field_fn(Protocol::GET, "z", |v: &Vec3| v.get().z)?;
    module.field_fn(Protocol::SET, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z = z;
    })?;
    module.field_fn(Protocol::ADD_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z += z;
    })?;
    module.field_fn(Protocol::SUB_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z -= z;
    })?;
    module.field_fn(Protocol::MUL_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z *= z;
    })?;
    module.field_fn(Protocol::DIV_ASSIGN, "z", |v: &mut Vec3, z: f32| {
        v.get_mut().z /= z;
    })?;

    Ok(module)
}
