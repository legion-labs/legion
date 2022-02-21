#![allow(unsafe_code)]

use lgn_core::BumpAllocatorHandle;
use lgn_math::{Mat4, Vec3};
use lgn_tracing::span_fn;
use std::sync::Mutex;

pub struct DebugDisplay {
    display_lists: Mutex<*const DisplayList>,
}

#[allow(clippy::mutex_atomic)]
impl DebugDisplay {
    pub fn new() -> Self {
        Self {
            display_lists: Mutex::new(std::ptr::null()),
        }
    }

    pub fn create_display_list<F: FnOnce(&mut DisplayListBuilder<'_>)>(
        &self,
        bump: &BumpAllocatorHandle,
        f: F,
    ) {
        let mut display_list = bump.alloc(DisplayList {
            primitives: std::ptr::null(),
            next: std::ptr::null(),
        });

        let mut builder = DisplayListBuilder { bump, display_list };
        f(&mut builder);
        let mut display_lists = self.display_lists.lock().unwrap();
        display_list.next = *display_lists;
        *display_lists = display_list;
    }

    #[span_fn]
    pub fn render_primitives<F: FnMut(&DebugPrimitive)>(&self, mut f: F) {
        let mut p_display_list = self.display_lists.lock().unwrap().to_owned();
        while !p_display_list.is_null() {
            let display_list = unsafe { &*p_display_list };
            let mut p_primitive = display_list.primitives;
            while !p_primitive.is_null() {
                let primitive = unsafe { &*p_primitive };
                f(primitive);
                p_primitive = primitive.next;
            }
            p_display_list = display_list.next;
        }
    }

    pub fn clear(&mut self) {
        *self.display_lists.lock().unwrap() = std::ptr::null();
    }
}

unsafe impl Send for DebugDisplay {}
unsafe impl Sync for DebugDisplay {}

impl Default for DebugDisplay {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DisplayListBuilder<'system> {
    bump: &'system BumpAllocatorHandle,
    display_list: &'system mut DisplayList,
}

impl<'system> DisplayListBuilder<'system> {
    pub fn add_mesh(&mut self, transform: Mat4, mesh_id: u32, color: Vec3) {
        let primitive = self.bump.alloc(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Mesh { mesh_id },
            transform,
            color,
            next: self.display_list.primitives,
        });
        self.display_list.primitives = primitive;
    }
}

pub struct DisplayList {
    primitives: *const DebugPrimitive,
    next: *const DisplayList,
}

pub enum DebugPrimitiveType {
    Mesh { mesh_id: u32 },
}

pub struct DebugPrimitive {
    pub primitive_type: DebugPrimitiveType,
    pub transform: Mat4,
    pub color: Vec3,
    next: *const DebugPrimitive,
}
