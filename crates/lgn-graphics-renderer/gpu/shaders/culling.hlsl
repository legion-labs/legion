#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/culling_pipeline_layout.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/transform.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"

float aabb_max_z(float4 aabb, float2 view_port, float debug_index) {
    float4 aabb_vp = min(aabb * view_port.xyxy, view_port.xyxy);
    float2 size = aabb_vp.zw - aabb_vp.xy;
    float lod = clamp(ceil(log2(max(size.x, size.y))) - 1, 0, push_constant.hzb_max_lod);
   
    float4 aabb_ts = float4(aabb_vp.x, view_port.y - aabb_vp.w, aabb_vp.z, view_port.y - aabb_vp.y);
    uint4 iaabb = (uint4)(aabb_ts * exp2(-lod));

    culling_debug[debug_index].aabb = aabb;
    culling_debug[debug_index].aabb_vp = aabb_vp;    
    culling_debug[debug_index].iaabb = iaabb;
    culling_debug[debug_index].lod = lod;

    float max_z = 0;
    for (uint i = iaabb.y; i <= iaabb.w; i++) {
        for (uint j = iaabb.x; j <= iaabb.z; j++) {
            float depth = hzb_texture.Load(uint3(j, i, lod));
            max_z = max(max_z, depth); 
        }
    }
    return max_z;
}

[numthreads(256, 1, 1)]
void main_cs(uint3 dt_id : SV_DispatchThreadID) {
    if (dt_id.x < gpu_instance_count[0]) {
        GpuInstanceData instance_data = gpu_instance_data[dt_id.x];
        
        uint va_table_address = va_table_address_buffer[instance_data.gpu_instance_id];
        GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, va_table_address);
        MeshDescription mesh_desc = LoadMeshDescription(static_buffer, addresses.mesh_description_va);

        Transform transform = LoadTransform(static_buffer, addresses.world_transform_va);
        float3 sphere_world_pos = transform_position(transform, mesh_desc.bounding_sphere.xyz);
       
        float bv_radius = mesh_desc.bounding_sphere.w * max(transform.scale.x, max(transform.scale.y, transform.scale.z));
        bool culled = false;

    #if FIRST_PASS
        if (push_constant.options.is_set(CullingOptions_GATHER_PERF_STATS)) {
            InterlockedAdd(culling_efficiency[0].total_elements, 1);
        }
        
        for (uint i = 0; i < 6 && !culled; i++) {
            float plane_test = dot(view_data.culling_planes[i], float4(sphere_world_pos, 1.0));

            if (plane_test - bv_radius > 0.0) {
                culled = true;
            }
        }
        if (!culled && push_constant.options.is_set(CullingOptions_GATHER_PERF_STATS)) {
            InterlockedAdd(culling_efficiency[0].frustum_visible, 1);
        }    
    #endif

        if (!culled) {
            float4 center_pos_view = float4(transform_position(view_data.camera_translation, view_data.camera_rotation, sphere_world_pos), 1.0);

            float4 min_view = center_pos_view + float4(-bv_radius, -bv_radius, 0.0, 0.0);
            float4 max_view = center_pos_view + float4(bv_radius, bv_radius, 0.0, 0.0);                                                          
            float4 closest_view = center_pos_view + float4(0.0, 0.0, -bv_radius, 0.0);    

            float4 min_proj = mul(view_data.projection, min_view);
            float4 max_proj = mul(view_data.projection, max_view);
            float4 closest_proj = mul(view_data.projection, closest_view);

            float4 aabb = clamp(float4(min_proj.xy / min_proj.w, max_proj.xy / max_proj.w), -1.0, 1.0) * 0.5 + 0.5;

            uint debug_index = dt_id.x;
            float max_z = aabb_max_z(aabb, push_constant.hzb_pixel_extents, debug_index);
            float depth = closest_proj.z / closest_proj.w;

            culling_debug[debug_index].gpu_instance = instance_data.gpu_instance_id;
            culling_debug[debug_index].depth = depth;
            culling_debug[debug_index].max_z = max_z;

            if (depth < 1.0 && depth > max_z) {
                culled = true;

            #if FIRST_PASS
                uint previous_count = 0;
                InterlockedAdd(culled_count[0], 1, previous_count);
    
                culled_args[0].yz = 1;
                InterlockedMax(culled_args[0].x, (previous_count + 256) / 256);

                culled_instances[previous_count] = instance_data;
            #endif
            }            
        }
        
        if (!culled) {
            if (push_constant.options.is_set(CullingOptions_GATHER_PERF_STATS)) {
                InterlockedAdd(culling_efficiency[0].occlusion_visible, 1);
            }
            
            uint first_pass = push_constant.first_render_pass;
            uint last_pass = first_pass + push_constant.num_render_passes;

            for (uint pass_idx = first_pass; pass_idx < last_pass; pass_idx++) {

                uint offset_base_va = render_pass_data[pass_idx].offset_base_va;
                uint offset_va = offset_base_va += (instance_data.state_id * 8);
        
                uint count_offset = static_buffer.Load<uint>(offset_va);
                uint indirect_arg_offset = static_buffer.Load<uint>(offset_va + 4);

                uint element_offset = 0;
                InterlockedAdd(draw_count[count_offset], 1, element_offset);
                uint inirect_offset = (indirect_arg_offset + element_offset) * 5;

                draw_args[inirect_offset + 0] = mesh_desc.index_count;
                draw_args[inirect_offset + 1] = 1;
                draw_args[inirect_offset + 2] = mesh_desc.index_offset;
                draw_args[inirect_offset + 3] = 0;
                draw_args[inirect_offset + 4] = instance_data.gpu_instance_id;
            }
        }
    }
}
