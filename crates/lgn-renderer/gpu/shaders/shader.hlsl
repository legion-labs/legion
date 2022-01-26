#include "crate://lgn-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/gpu_instance_transform.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/gpu_instance_color.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/gpu_instance_picking_data.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-renderer/gpu/include/brdf.hsh"

struct GpuPipelineVertexIn
{
    uint vertexId: SV_VertexID;
    uint instanceId: SV_InstanceID;
    uint va_table_address: INSTANCE0;
};

struct VertexIn {
    float4 pos : POSITION;
    float4 normal : NORMAL;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 normal : NORMAL;
    float3 pos : POSITION;
    nointerpolation uint va_table_address: INSTANCE0;
};

VertexOut main_vs(GpuPipelineVertexIn vertexIn) {
    GpuInstanceVATable addresses = static_buffer.Load<GpuInstanceVATable>(vertexIn.va_table_address);

    VertexIn vertex_in = static_buffer.Load<VertexIn>(addresses.vertex_buffer_va + vertexIn.vertexId * 56);
    VertexOut vertex_out;

    GpuInstanceTransform transform = static_buffer.Load<GpuInstanceTransform>(addresses.world_transform_va);
    float4x4 world = transpose(transform.world);

    float4 pos_view_relative = mul(view_data.view, mul(world, vertex_in.pos));
    
    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    vertex_out.normal = mul(view_data.view, mul(world, vertex_in.normal)).xyz;
    vertex_out.va_table_address = vertexIn.va_table_address;

    return vertex_out;
}

Lighting CalculateIncidentDirectionalLight(DirectionalLight light, float3 pos, float3 normal, MaterialData material, float3 albedo) {
    float3 light_dir = normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material, albedo);
    }

    lighting.diffuse *= light.color * light.radiance;
    lighting.specular *= light.color * light.radiance;

    return lighting;
}

Lighting CalculateIncidentOmniDirectionalLight(OmniDirectionalLight light, float3 pos, float3 normal, MaterialData material, float3 albedo) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material, albedo);
    }

    lighting.diffuse *= light.color * light.radiance / distance;
    lighting.specular *= light.color * light.radiance / distance;

    return lighting;
}

Lighting CalculateIncidentSpotLight(SpotLight light, float3 pos, float3 normal, MaterialData material, float3 albedo) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material, albedo);
    }

    float cos_between_dir = dot(normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz), light_dir);
    float cos_half_angle = cos(light.cone_angle/2.0);
    float diff = 1.0 - cos_half_angle;
    float factor = saturate((cos_between_dir - cos_half_angle)/diff);

    lighting.diffuse *= factor * light.color * light.radiance / distance;
    lighting.specular *= factor * light.color * light.radiance / distance;

    return lighting;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    GpuInstanceVATable addresses = static_buffer.Load<GpuInstanceVATable>(vertex_out.va_table_address);

    MaterialData material = static_buffer.Load<MaterialData>(addresses.material_data_va);
    GpuInstanceColor instance_color = static_buffer.Load<GpuInstanceColor>(addresses.instance_color_va);

    float3 albedo = lerp(material.base_albedo.xyz, instance_color.color.xyz, instance_color.color_blend); 

    float3 ambient_color = albedo * lighting_data.ambient_reflection;
    float3 diffuse_color = lighting_data.diffuse_reflection.xxx;
    float3 spec_color = lighting_data.specular_reflection.xxx;

    float3 color = ambient_color;
    for (uint i = 0; i < lighting_data.num_directional_lights; i++)
    {
        DirectionalLight light = directional_lights[i];
        Lighting lighting = CalculateIncidentDirectionalLight(light, vertex_out.pos, vertex_out.normal, material, albedo);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    for (i = 0; i < lighting_data.num_omni_directional_lights; i++)
    {
        OmniDirectionalLight light = omni_directional_lights[i];
        Lighting lighting = CalculateIncidentOmniDirectionalLight(light, vertex_out.pos, vertex_out.normal, material, albedo);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    for (i = 0; i < lighting_data.num_spot_lights; i++)
    {
        SpotLight light = spot_lights[i];
        Lighting lighting = CalculateIncidentSpotLight(light, vertex_out.pos, vertex_out.normal, material, albedo);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    return float4(color, 1.0);
}
