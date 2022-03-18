#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/picking_pipeline_layout.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_transform.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_color.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_picking_data.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float4 picked_world_pos : COLOR;
    nointerpolation uint va_table_address: INSTANCE0;
};

VertexOut main_vs(GpuPipelineVertexIn vertexIn) {
    VertexIn vertex_in = (VertexIn)0;
    VertexOut vertex_out = (VertexOut)0;
    float4x4 world = push_constant.world;

    if (push_constant.use_gpu_pipeline) {
        GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertexIn.va_table_address);
        MeshDescription mesh_desc = LoadMeshDescription(static_buffer, addresses.mesh_description_va);
        
        vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexIn.vertexId);

        GpuInstanceTransform transform = LoadGpuInstanceTransform(static_buffer, addresses.world_transform_va);
        world = transform.world;        
    }
    else
    {
        MeshDescription mesh_desc = LoadMeshDescription(static_buffer, push_constant.mesh_description_offset);
        vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexIn.vertexId);
    }

    float4 world_pos = mul(world, float4(vertex_in.pos, 1.0));
    vertex_out.hpos = mul(view_data.projection_view, world_pos);

    vertex_out.picked_world_pos = world_pos;
    vertex_out.va_table_address = vertexIn.va_table_address;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET 
{
    uint pickingId = push_constant.picking_id;
    if (push_constant.use_gpu_pipeline) 
    {
        GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertex_out.va_table_address);
        pickingId = LoadGpuInstancePickingData(static_buffer, addresses.picking_data_va).picking_id;
    }

    float2 picking_pos = vertex_out.hpos * view_data.pixel_size.zw * view_data.logical_size.xy;
    float2 proximity = picking_pos.xy - view_data.cursor_pos;

    if (dot(proximity, proximity) < push_constant.picking_distance)
    {
        uint write_index = 0;
        InterlockedAdd(picked_count[0], 1, write_index);

        picked_objects[write_index].picking_pos = float4(vertex_out.picked_world_pos.xyz, vertex_out.hpos.z);
        picked_objects[write_index].picking_id = pickingId;
    }

    return float4(proximity.xy, picking_pos.xy);
}
