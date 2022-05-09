#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/const_color_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/transform.hsh"

struct VertexOut {
    float4 hpos : SV_POSITION;
    float4 color: COLOR;
};

VertexOut main_vs(uint vertexId: SV_VertexID) {
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, push_constant.mesh_description_offset);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, push_constant.mesh_description_offset, vertexId);
    VertexOut vertex_out;

    float3 view_pos = transform_from_tr(view_data.camera_translation, view_data.camera_rotation)
        .apply_to_point(
            transform_from_data(push_constant.transform)
            .apply_to_point(vertex_in.pos)
        );

    vertex_out.hpos = mul(view_data.projection, float4(view_pos, 1.0));
    vertex_out.color = vertex_in.color;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return lerp(vertex_out.color, unpack_linear(push_constant.color), push_constant.color_blend); 
}
