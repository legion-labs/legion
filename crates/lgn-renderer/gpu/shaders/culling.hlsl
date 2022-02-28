#include "crate://lgn-renderer/gpu/pipeline_layout/culling_pipeline_layout.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-renderer/gpu/include/mesh.hsh"

[numthreads(256, 1, 1)]
void main_cs(uint3 dt_id : SV_DispatchThreadID) {
    if (dt_id.x < push_constant.num_gpu_instances) {
        GpuInstanceData instance_data = gpu_instance_data[dt_id.x];
        
        for (uint pass_idx = 0; pass_idx < push_constant.num_render_passes; pass_idx++) {

            uint offset_base_va = render_pass_data[pass_idx].offset_base_va;
            uint offset_va = offset_base_va += (instance_data.material_id * 8);
            
            uint count_offset = static_buffer.Load<uint>(offset_va);
            uint indirect_arg_offset = static_buffer.Load<uint>(offset_va + 4);

            uint element_offset = 0;
            InterlockedAdd(count_buffer[count_offset], 1, element_offset);
            uint inirect_offset = (indirect_arg_offset + element_offset) * 5;

            uint va_table_address = va_table_address_buffer[instance_data.gpu_instance_id];
            GpuInstanceVATable addresses = static_buffer.Load<GpuInstanceVATable>(va_table_address);
            MeshDescription mesh_desc = static_buffer.Load<MeshDescription>(addresses.mesh_description_va);

            if( mesh_desc.attrib_mask.is_set(MeshAttribMask_INDEX)) {
                indirect_arg_buffer[inirect_offset + 0] = mesh_desc.index_count;
                indirect_arg_buffer[inirect_offset + 1] = 1;
                //indirect_arg_buffer[inirect_offset + 2] = mesh_desc.index_offset;
                indirect_arg_buffer[inirect_offset + 2] = 0;
                indirect_arg_buffer[inirect_offset + 3] = instance_data.gpu_instance_id;
            }
            else {
                indirect_arg_buffer[inirect_offset + 0] = mesh_desc.vertex_count;
                indirect_arg_buffer[inirect_offset + 1] = 1;
                indirect_arg_buffer[inirect_offset + 2] = 0;
                indirect_arg_buffer[inirect_offset + 3] = instance_data.gpu_instance_id;
            }
        }
    }
}
