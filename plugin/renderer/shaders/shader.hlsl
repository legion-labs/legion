#include "crate://renderer/codegen/hlsl/cgen_type/omnidirectional_light.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/directional_light.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/spotlight.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/view_data.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/instance_push_constant_data.hlsl"

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

struct LightingManager {
    uint num_directional_lights;
    uint num_omnidirectional_lights;
    uint num_spotlights;
    bool diffuse;
    bool specular;
    float specular_reflection;
    float diffuse_reflection;
    float ambient_reflection;
    float shininess;
};

struct EntityTransforms {
    float4x4 world;
};


ConstantBuffer<ViewData> view_data;
ConstantBuffer<LightingManager> lighting_manager;
ByteAddressBuffer static_buffer;
[[vk::push_constant]]
ConstantBuffer<InstancePushConstantData> instance_data;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(instance_data.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    EntityTransforms transform = static_buffer.Load<EntityTransforms>(instance_data.world_offset);
    float4x4 world = transpose(transform.world);

    float4 pos_view_relative = mul(view_data.view, mul(world, vertex_in.pos));
    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    vertex_out.normal = mul(view_data.view, mul(world, vertex_in.normal)).xyz;
    return vertex_out;
}

#define PI 3.141592

struct Lighting {
    float3 specular;
    float3 diffuse;
};

StructuredBuffer<DirectionalLight> directional_lights;
StructuredBuffer<OmnidirectionalLight> omnidirectional_lights;
StructuredBuffer<Spotlight> spotlights;

// Position, normal, and light direction are in view space
float GetSpecular(float3 pos, float3 light_dir, float3 normal) {
    float3 view_dir = normalize(-pos);
    float3 light_reflect = normalize(reflect(-light_dir, normal));
    return pow(saturate(dot(view_dir, light_reflect)), lighting_manager.shininess);
}

Lighting CalculateIncidentDirectionalLight(DirectionalLight light, float3 normal, float3 pos) {
    float3 light_dir = normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        specular = GetSpecular(pos, light_dir, normal);
    }

    Lighting lighting;
    lighting.diffuse = lambertian * light.color * light.radiance;
    lighting.specular = specular * light.color * light.radiance;

    return lighting;
}

Lighting CalculateIncidentOmnidirectionalLight(OmnidirectionalLight light, float3 normal, float3 pos) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        specular = GetSpecular(pos, light_dir, normal);;
    }

    Lighting lighting;
    lighting.diffuse = lambertian * light.color * light.radiance / distance;
    lighting.specular = specular * light.color * light.radiance / distance;

    return lighting;
}

Lighting CalculateIncidentSpotlight(Spotlight light, float3 normal, float3 pos) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        specular = GetSpecular(pos, light_dir, normal);
    }

    float cos_between_dir = dot(normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz), light_dir);
    float cos_half_angle = cos(light.cone_angle/2.0);
    float diff = 1.0 - cos_half_angle;
    float factor = saturate((cos_between_dir - cos_half_angle)/diff);
    
    Lighting lighting;
    lighting.diffuse = factor * lambertian * light.color * light.radiance / distance;
    lighting.specular = factor * specular * light.color * light.radiance / distance;

    return lighting;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float3 uniform_color = instance_data.color.xyz; 
    float3 ambient_color = uniform_color * lighting_manager.ambient_reflection;
    float3 diffuse_color = uniform_color * lighting_manager.diffuse_reflection;
    float3 spec_color = float3(1.0, 1.0, 1.0) * lighting_manager.specular_reflection;

    float3 color = ambient_color;
    for (uint i = 0; i < lighting_manager.num_directional_lights; i++)
    {
        DirectionalLight light = directional_lights[i];
        Lighting lighting = CalculateIncidentDirectionalLight(light, vertex_out.normal, vertex_out.pos);
        if (lighting_manager.diffuse)
        {
            color += diffuse_color * lighting.diffuse;
        }

        if (lighting_manager.specular)
        {
            color += spec_color * lighting.specular;
        }
    }

    for (i = 0; i < lighting_manager.num_omnidirectional_lights; i++)
    {
        OmnidirectionalLight light = omnidirectional_lights[i];
        Lighting lighting = CalculateIncidentOmnidirectionalLight(light, vertex_out.normal, vertex_out.pos);
        if (lighting_manager.diffuse)
        {
            color += diffuse_color * lighting.diffuse;
        }

        if (lighting_manager.specular)
        {
            color += spec_color * lighting.specular;
        }
    }

    for (i = 0; i < lighting_manager.num_spotlights; i++)
    {
        Spotlight light = spotlights[i];
        Lighting lighting = CalculateIncidentSpotlight(light, vertex_out.normal, vertex_out.pos);
        if (lighting_manager.diffuse)
        {
            color += diffuse_color * lighting.diffuse;
        }

        if (lighting_manager.specular)
        {
            color += spec_color * lighting.specular;
        }
    }

    float4 result = float4(color, 1.0);
    float4 picking_color = float4(0.0f, 0.5f, 0.5f, 1.0f);

    if (instance_data.is_picked != 0)
        result = result * 0.25f + picking_color * 0.75f;

    return result;
}