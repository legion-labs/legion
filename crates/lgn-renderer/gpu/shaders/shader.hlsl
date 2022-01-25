#include "crate://lgn-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/entity_transforms.hlsl"

#include "crate://lgn-renderer/gpu/include/brdf.hlsl"

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
};

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    EntityTransforms transform = static_buffer.Load<EntityTransforms>(push_constant.world_offset);
    float4x4 world = transpose(transform.world);

    float4 pos_view_relative = mul(view_data.view, mul(world, vertex_in.pos));
    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;

    vertex_out.normal = mul(view_data.view, mul(world, vertex_in.normal)).xyz;

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
    MaterialData material = static_buffer.Load<MaterialData>(push_constant.material_offset);

    float3 albedo = lerp(material.base_albedo.xyz, push_constant.color.xyz, push_constant.color_blend); 

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

    float4 result = float4(color, 1.0);
    float4 picking_color = float4(0.0f, 0.5f, 0.5f, 1.0f);

    if (push_constant.is_picked != 0)
        result = result * 0.25f + picking_color * 0.75f;

    return result;
}