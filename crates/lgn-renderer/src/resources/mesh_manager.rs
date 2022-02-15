use lgn_ecs::prelude::{Local, Query, Res, ResMut, Without};
use lgn_graphics_api::PagedBufferAllocation;

use super::{GpuUniformData, GpuUniformDataContext, UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{cgen, static_mesh_render_data::StaticMeshRenderData, Renderer};
    components::ManipulatorComponent,
    egui::egui_plugin::Egui,

pub struct MeshManager {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<StaticMeshRenderData>,
    mesh_description_offsets: Vec<u32>,
    allocations: Vec<PagedBufferAllocation>,
}

impl Drop for MeshManager {
    fn drop(&mut self) {
        while let Some(allocation) = self.allocations.pop() {
            self.static_buffer.free_segment(allocation);
        }
    }
}

pub enum DefaultMeshType {
    Plane = 0,
    Cube,
    Pyramid,
    WireframeCube,
    GroundPlane,
    Torus,
    Cone,
    Cylinder,
    Sphere,
    Arrow,
    RotationRing,
}

impl MeshManager {
    pub fn new(renderer: &Renderer) -> Self {
        let static_buffer = renderer.static_buffer().clone();

        let mut mesh_manager = Self {
            static_buffer,
            static_meshes: Vec::new(),
            mesh_description_offsets: Vec::new(),
            allocations: Vec::new(),
        };

        // Keep consistent with DefaultMeshType
        let default_meshes = vec![
            StaticMeshRenderData::new_plane(1.0),
            StaticMeshRenderData::new_cube(0.5),
            StaticMeshRenderData::new_pyramid(0.5, 1.0),
            StaticMeshRenderData::new_wireframe_cube(1.0),
            StaticMeshRenderData::new_ground_plane(6, 5, 0.25),
            StaticMeshRenderData::new_torus(0.1, 32, 0.5, 128),
            StaticMeshRenderData::new_cone(0.25, 1.0, 32),
            StaticMeshRenderData::new_cylinder(0.25, 1.0, 32),
            StaticMeshRenderData::new_sphere(0.25, 64, 64),
            StaticMeshRenderData::new_arrow(),
            StaticMeshRenderData::new_torus(0.01, 8, 0.5, 128),
        ];

        mesh_manager.add_meshes(renderer, default_meshes);
        mesh_manager
    }

    pub fn add_meshes(&mut self, renderer: &Renderer, mut meshes: Vec<StaticMeshRenderData>) {
        if meshes.is_empty() {
            return;
        }
        let mut vertex_data_size_in_bytes = 0;
        for mesh in &meshes {
            vertex_data_size_in_bytes += u64::from(mesh.size_in_bytes())
                + std::mem::size_of::<cgen::cgen_type::MeshDescription>() as u64;
        }

        let static_allocation = self
            .static_buffer
            .allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut static_mesh_descs = Vec::with_capacity(meshes.len());
        let mut offset = static_allocation.offset();

        for mesh in &meshes {
            let (new_offset, mesh_desc) = mesh.make_gpu_update_job(&mut updater, offset as u32);
            static_mesh_descs.push(mesh_desc);
            offset = u64::from(new_offset);
        }

        let mut mesh_description_offsets = Vec::with_capacity(meshes.len());
        updater.add_update_jobs(&static_mesh_descs, offset);
        for (i, _) in static_mesh_descs.into_iter().enumerate() {
            mesh_description_offsets.push(
                offset as u32
                    + (i * std::mem::size_of::<cgen::cgen_type::MeshDescription>()) as u32,
            );
        }

        renderer.add_update_job_block(updater.job_blocks());
        self.static_meshes.append(&mut meshes);
        self.mesh_description_offsets
            .append(&mut mesh_description_offsets);
        self.allocations.push(static_allocation);
    }

    pub fn mesh_description_offset_from_id(&self, mesh_id: u32) -> u32 {
        if mesh_id < self.mesh_description_offsets.len() as u32 {
            self.mesh_description_offsets[mesh_id as usize]
        } else {
            0
        }
    }

    pub fn mesh_from_id(&self, mesh_id: u32) -> &StaticMeshRenderData {
        &self.static_meshes[mesh_id as usize]
    }

    pub fn mesh_indices_from_id(&self, mesh_id: u32) -> &Option<Vec<u32>> {
        &self.static_meshes[mesh_id as usize].indices
    }

    pub fn max_id(&self) -> usize {
        self.static_meshes.len()
    }
}

pub struct MeshManagerUIState {
    path: String,
}

impl Default for MeshManagerUIState {
    fn default() -> Self {
        Self {
            path: String::from(
                //"C:/work/glTF-Sample-Models/2.0/FlightHelmet/glTF/FlightHelmet.gltf",
                //"C:/work/glTF-Sample-Models/sourceModels/DragonAttenuation/Dragon_Attenuation.blend",
                "C:/work/glTF-Sample-Models/2.0/DragonAttenuation/glTF/DragonAttenuation.gltf",
            ),
        }
    }
}

//#[allow(clippy::needless_pass_by_value)]
//pub fn ui_mesh_manager(
//    egui_ctx: Res<'_, Egui>,
//    _renderer: Res<'_, Renderer>,
//    mut mesh_manager: ResMut<'_, MeshManager>,
//    mut ui_state: Local<'_, MeshManagerUIState>,
//    mut q_static_meshes: Query<'_, '_, &mut StaticMesh, Without<ManipulatorComponent>>,
//    uniform_data: Res<'_, GpuUniformData>,
//) {
//    let data_context = GpuUniformDataContext::new(&uniform_data);
//
//    egui::Window::new("Mesh manager").show(&egui_ctx.ctx, |ui| {
//        ui.add(egui::text_edit::TextEdit::singleline(&mut ui_state.path));
//        //if ui.small_button("Load mesh (gltf)").clicked() {
//        //    mesh_manager.add_meshes(
//        //        renderer.as_ref(),
//        //        StaticMeshRenderData::new_gltf(ui_state.path.clone()),
//        //    );
//        //}
//        for (idx, mut mesh) in q_static_meshes.iter_mut().enumerate() {
//            let selected_text = format!("Mesh ID {}", mesh.mesh_id);
//            let mut selected_idx = mesh.mesh_id;
//            egui::ComboBox::from_label(format!("Mesh {}", idx))
//                .selected_text(selected_text)
//                .show_ui(ui, |ui| {
//                    for mesh_id in 0..mesh_manager.max_id() {
//                        ui.selectable_value(
//                            &mut selected_idx,
//                            mesh_id as usize,
//                            format!("Mesh ID {}", mesh_id),
//                        );
//                    }
//                });
//            if selected_idx != mesh.mesh_id {
//                mesh.set_mesh_id(&mesh_manager, selected_idx);
//            }
//        }
//    });
//}
