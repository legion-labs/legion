#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/transform.hsh"

struct VertexOut {  
    float4 hpos : SV_POSITION;
};

VertexOut main_vs(GpuPipelineVertexIn vertexIn) {
    GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertexIn.va_table_address);
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, addresses.mesh_description_va);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, addresses.mesh_description_va, vertexIn.vertexId);
    VertexOut vertex_out;

    TransformData transform = LoadTransformData(static_buffer, addresses.world_transform_va);
    float3 world_pos = transform_from_data(transform).apply_to_point(vertex_in.pos);
    float3 view_pos = transform_from_tr(view_data.camera_translation, view_data.camera_rotation).apply_to_point(world_pos);

    vertex_out.hpos = mul(view_data.projection, float4(view_pos, 1.0));

    return vertex_out;
}
