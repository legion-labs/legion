struct VertexIn {
    float4 pos : POSITION;
    float4 normal : NORMAL;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 picking_pos : POSITION;
};

struct ConstData {
    float4x4 view_proj;
    float4x4 inv_view_proj;
    float4 screen_size;
    float2 cursor_pos;
    float picking_distance;
};

struct EntityTransforms {
    float4x4 world;
};

ConstantBuffer<ConstData> const_data;
ByteAddressBuffer static_buffer;

struct PickingData
{
    float3 picking_pos;
    uint picking_id;
};

RWStructuredBuffer<uint> picked_count;
RWStructuredBuffer<PickingData> picked_objects;

struct PushConstData {
    uint vertex_offset;
    uint world_offset;
    uint picking_id;
};

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    EntityTransforms transform = static_buffer.Load<EntityTransforms>(push_constant.world_offset);
    float4x4 world = transpose(transform.world);

    float4 world_pos = mul(world, vertex_in.pos);
    vertex_out.hpos = mul(const_data.view_proj, world_pos);

    float2 pers_div = vertex_out.hpos.xy / vertex_out.hpos.w;
    pers_div.y *= -1.0f;

    vertex_out.picking_pos = float3((pers_div + 1.0f) * 0.5f * const_data.screen_size.xy, world_pos.z);

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET 
{
    float2 proximity = vertex_out.picking_pos.xy - const_data.cursor_pos;

    if (dot(proximity, proximity) < const_data.picking_distance)
    {
        uint write_index = 0;
        InterlockedAdd(picked_count[0], 1, write_index);

        picked_objects[write_index].picking_pos = vertex_out.picking_pos;
        picked_objects[write_index].picking_id = push_constant.picking_id;
    }

    return float4(proximity.xy, dot(proximity, proximity), 1.0f);
}
