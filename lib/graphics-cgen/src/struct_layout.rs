use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct StructMemberLayout {
    pub offset: u32,
    pub absolute_offset: u32,
    pub size: u32,
    pub padded_size: u32,    
    pub array_stride: u32,    
}

#[derive(Debug, PartialEq)]
pub struct StructLayout {
    pub size: u32,
    pub padded_size: u32,
    pub members: Vec<StructMemberLayout>,
}

pub struct StructLayouts {
    layout_map: HashMap<u32, StructLayout>,
}

impl StructLayouts {
    pub fn new() -> Self {
        Self {
            layout_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u32, layout: StructLayout) {
        self.layout_map.insert(id, layout);
    }

    pub fn get(&self, id: u32) -> Option<&StructLayout> {
        self.layout_map.get(&id)
    }
}
