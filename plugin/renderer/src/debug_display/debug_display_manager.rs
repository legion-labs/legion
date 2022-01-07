#![allow(unsafe_code)]

use std::sync::Mutex;

use bumpalo::Bump;
use lgn_math::Vec3;

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
    Cube,
    Arrow { dir: Vec3 },
}

pub struct DebugPrimitive {
    pub primitive_type: DebugPrimitiveType,
    pub pos: Vec3,
}

#[derive(Default)]
pub struct DisplayList {
    primitives: Vec<*mut DebugPrimitive>,
}

impl DisplayList {
    pub fn add_cube(&mut self, pos: Vec3, bump: &Bump) {
        self.primitives.push(bump.alloc(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Cube,
            pos,
        }));
    }
    pub fn add_arrow(&mut self, start: Vec3, end: Vec3, bump: &Bump) {
        self.primitives.push(bump.alloc(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Arrow { dir: end - start },
            pos: start,
        }));
    }
}
