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

    pub fn cubes(&mut self) -> Vec<Cube> {
        let mut cubes = Vec::new();
        for display_list in &mut self.display_lists {
            cubes.append(&mut display_list.cubes);
        }
        cubes
    }

    pub fn clear_display_lists(&mut self) {
        self.display_lists.clear();
    }
}

pub struct Cube {
    pub pos: Vec3,
}

#[derive(Default)]
pub struct DisplayList {
    cubes: Vec<Cube>,
}

impl DisplayList {
    pub fn add_cube(&mut self, pos: Vec3) {
        self.cubes.push(Cube { pos });
    }
    pub fn add_sphere(&mut self) {}
}
