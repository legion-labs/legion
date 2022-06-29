#![allow(unsafe_code)]

use lgn_graphics_data::Color;
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;
use std::sync::Mutex;

use crate::resources::{DefaultMeshType, RenderMeshId};

pub struct DebugDisplay {
    display_lists: Mutex<Vec<DisplayList>>,
}

#[allow(clippy::mutex_atomic)]
impl DebugDisplay {
    pub fn new() -> Self {
        Self {
            display_lists: Mutex::new(vec![]),
        }
    }

    pub fn create_display_list<F: FnOnce(&mut DisplayListBuilder<'_>)>(&self, f: F) {
        let mut display_list = DisplayList { primitives: vec![] };
        let mut builder = DisplayListBuilder {
            display_list: &mut display_list,
        };
        f(&mut builder);
        let mut display_lists = self.display_lists.lock().unwrap();
        display_lists.push(display_list);
    }

    #[span_fn]
    pub fn render_primitives<F: FnMut(&DebugPrimitive)>(&self, mut f: F) {
        let display_lists = self.display_lists.lock().unwrap();
        for display_list in display_lists.iter() {
            for primitive in &display_list.primitives {
                f(primitive);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.display_lists.lock().unwrap().clear();
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
    display_list: &'system mut DisplayList,
}

impl<'system> DisplayListBuilder<'system> {
    pub fn add_draw_call(
        &mut self,
        transform: &GlobalTransform,
        primitive_type: DebugPrimitiveType,
        color: Color,
        material: DebugPrimitiveMaterial,
    ) {
        let primitive = DebugPrimitive {
            primitive_type,
            transform: *transform,
            color,
            material,
        };
        self.display_list.primitives.push(primitive);
    }
}

pub struct DisplayList {
    primitives: Vec<DebugPrimitive>,
}

pub enum DebugPrimitiveType {
    Mesh { mesh_id: RenderMeshId },
    DefaultMesh { default_mesh_type: DefaultMeshType },
}

impl DebugPrimitiveType {
    pub fn mesh(mesh_id: RenderMeshId) -> Self {
        Self::Mesh { mesh_id }
    }

    pub fn default_mesh(default_mesh_type: DefaultMeshType) -> Self {
        Self::DefaultMesh { default_mesh_type }
    }
}

#[allow(clippy::enum_variant_names)]
pub enum DebugPrimitiveMaterial {
    WireDepth,
    SolidDepth,
    WireNoDepth,
    SolidNoDepth,
}

pub struct DebugPrimitive {
    pub primitive_type: DebugPrimitiveType,
    pub transform: GlobalTransform,
    pub color: Color,
    pub material: DebugPrimitiveMaterial,
}
