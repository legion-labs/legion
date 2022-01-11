use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct StructMemberLayout {
    pub offset: u32,    
    pub padded_size: u32,
    pub array_stride: u32,
}

#[derive(Debug, PartialEq)]
pub struct StructLayout {    
    pub padded_size: u32,
    pub members: Vec<StructMemberLayout>,
}

#[derive(Default)]
pub struct StructLayouts {
    layout_map: HashMap<u32, StructLayout>,
}

impl StructLayouts {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, id: u32, layout: StructLayout) {
        self.layout_map.insert(id, layout);
    }

    pub fn get(&self, id: u32) -> Option<&StructLayout> {
        self.layout_map.get(&id)
    }
}
