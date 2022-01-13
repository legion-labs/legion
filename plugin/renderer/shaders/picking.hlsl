#include "crate://renderer/codegen/hlsl/cgen_type/view_data.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/const_data.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/picking_push_constant_data.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/picking_data.hlsl"

struct VertexIn {
    float4 pos : POSITION;
    float4 normal : NORMAL;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 picking_pos : POSITION;
    float3 picked_world_pos : COLOR;
};

struct EntityTransforms {
    float4x4 world;
};

ConstantBuffer<ViewData> view_data;
ConstantBuffer<ConstData> const_data;
ByteAddressBuffer static_buffer;
RWStructuredBuffer<uint> picked_count;
RWStructuredBuffer<PickingData> picked_objects;

[[vk::push_constant]]
ConstantBuffer<PickingPushConstantData> push_constant;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    float4x4 world = const_data.world;
    if (push_constant.world_offset != 0xFFFFFFFF)
    {
        EntityTransforms transform = static_buffer.Load<EntityTransforms>(push_constant.world_offset);
        world = transpose(transform.world);
    }

    float4 world_pos = mul(world, vertex_in.pos);
    vertex_out.hpos = mul(view_data.projection_view, world_pos);

    float2 pers_div = vertex_out.hpos.xy / vertex_out.hpos.w;
    pers_div.y *= -1.0f;

    vertex_out.picked_world_pos = world_pos.xyz;
    vertex_out.picking_pos = float3((pers_div + 1.0f) * 0.5f * view_data.screen_size.xy, world_pos.z);

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET 
{
    float2 proximity = vertex_out.picking_pos.xy - view_data.cursor_pos;

    if (dot(proximity, proximity) < const_data.picking_distance)
    {
        uint write_index = 0;
        InterlockedAdd(picked_count[0], 1, write_index);

        picked_objects[write_index].picking_pos = vertex_out.picked_world_pos;
        picked_objects[write_index].picking_id = push_constant.picking_id;
    }

    return float4(proximity.xy, dot(proximity, proximity), 1.0f);
}
