#![allow(unsafe_code)]

use std::sync::Mutex;

use bumpalo::Bump;
use lgn_math::Mat4;

struct DisplayListWrapper {
    ptr: *mut DisplayList,
}

unsafe impl Send for DisplayListWrapper {}

#[derive(Default)]
pub struct DebugDisplay {
    display_lists: Mutex<Vec<DisplayListWrapper>>,
}

impl DebugDisplay {
    pub fn create_display_list<F: FnOnce(&mut DisplayList)>(&mut self, bump: &Bump, f: F) {
        let display_list = bump.alloc(DisplayList::default());
        {
            let mut display_lists = self.display_lists.lock().unwrap();
            display_lists.push(DisplayListWrapper { ptr: display_list });
            f(display_list);
        }
    }

    pub fn render_primitives<F: FnMut(&DebugPrimitive)>(&mut self, mut f: F) {
        for display_list in self.display_lists.lock().unwrap().as_slice() {
            for primitive in unsafe { &(*display_list.ptr).primitives } {
                f(unsafe { &**primitive });
            }
        }
    }

    pub fn clear_display_lists(&mut self) {
        self.display_lists.lock().unwrap().clear();
    }
}

pub enum DebugPrimitiveType {
    Mesh { mesh_id: u32 },
}

pub struct DebugPrimitive {
    pub primitive_type: DebugPrimitiveType,
    pub transform: Mat4,
    pub color: (f32, f32, f32),
}

#[derive(Default)]
pub struct DisplayList {
    primitives: Vec<*mut DebugPrimitive>,
}

impl DisplayList {
    pub fn add_mesh(&mut self, transform: Mat4, mesh_id: u32, color: (f32, f32, f32), bump: &Bump) {
        self.primitives.push(bump.alloc(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Mesh { mesh_id },
            transform,
            color,
        }));
    }
}
