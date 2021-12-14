use lgn_math::Vec3;

#[derive(Default)]
pub struct DebugDisplay {
    display_lists: Vec<DisplayList>,
}

impl DebugDisplay {
    pub fn create_display_list(&mut self) -> &mut DisplayList {
        self.display_lists.push(DisplayList::default());
        self.display_lists.last_mut().unwrap()
    }

    pub fn primitives(&mut self) -> Vec<DebugPrimitive> {
        let mut primitives = Vec::new();
        for display_list in &mut self.display_lists {
            primitives.append(&mut display_list.primitives);
        }
        primitives
    }

    pub fn clear_display_lists(&mut self) {
        self.display_lists.clear();
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
    primitives: Vec<DebugPrimitive>,
}

impl DisplayList {
    pub fn add_cube(&mut self, pos: Vec3) {
        self.primitives.push(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Cube,
            pos,
        });
    }
    pub fn add_arrow(&mut self, start: Vec3, end: Vec3) {
        self.primitives.push(DebugPrimitive {
            primitive_type: DebugPrimitiveType::Arrow { dir: end - start },
            pos: start,
        });
    }
}
