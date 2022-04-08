#![allow(unsafe_code)]

use lgn_core::BumpAllocatorHandle;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;
use std::sync::Mutex;

use crate::resources::DefaultMeshType;

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

    pub fn end_frame(&mut self) {
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
    pub fn add_default_mesh(
        &mut self,
        transform: &GlobalTransform,
        default_mesh_type: DefaultMeshType,
        color: Color,
    ) {
        let primitive = self.bump.alloc(DebugPrimitive {
            primitive_type: DebugPrimitiveType::DefaultMesh { default_mesh_type },
            transform: *transform,
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
    // TODO(vdbdd): add those new types
    // DisplayList
    // Mesh
    DefaultMesh { default_mesh_type: DefaultMeshType },
}

pub struct DebugPrimitive {
    pub primitive_type: DebugPrimitiveType,
    pub transform: GlobalTransform,
    pub color: Color,
    next: *const DebugPrimitive,
}
