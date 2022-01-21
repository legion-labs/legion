#include "crate://renderer/codegen/hlsl/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://renderer/codegen/hlsl/cgen_type/entity_transforms.hlsl"

#include "crate://renderer/shaders/brdf.hlsl"

struct VertexIn {
    float4 pos : POSITION;
    float4 normal : NORMAL;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 normal : NORMAL;
    float3 tangent : TANGENT;
    float3 pos : POSITION;
};

struct Lighting {
    float3 specular;
    float3 diffuse;
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
    
    if (vertex_out.normal.x != vertex_out.normal.y || vertex_out.normal.x != vertex_out.normal.z)
        vertex_out.tangent = float3(vertex_out.normal.z - vertex_out.normal.y, vertex_out.normal.x - vertex_out.normal.z, vertex_out.normal.y - vertex_out.normal.x);  //(1,1,1)x vertex_out.normal
    else
        vertex_out.tangent = float3(vertex_out.normal.z - vertex_out.normal.y, vertex_out.normal.x + vertex_out.normal.z, -vertex_out.normal.y - vertex_out.normal.x);  //(-1,1,1)x vertex_out.normal
    vertex_out.tangent = normalize(vertex_out.tangent);
    //vertex_out.tangent = mul(view_data.view, mul(world, vertex_in.tangent)).xyz;

    return vertex_out;
}

float3 CalculateIncidentDirectionalLight(DirectionalLight light, float3 pos, float3 normal, float3 tangent, float3 binormal, MaterialData material) {
    float3 light_dir = normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz);
    float3 view_dir = normalize(-pos);

    return BRDF(light_dir, view_dir, normal, tangent, binormal, material) * light.color * light.radiance;
}

float3 CalculateIncidentOmniDirectionalLight(OmniDirectionalLight light, float3 pos, float3 normal, float3 tangent, float3 binormal, MaterialData material) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);
    float3 view_dir = normalize(-pos);

    return BRDF(light_dir, view_dir, normal, tangent, binormal, material) * light.color * light.radiance / distance;
}

float3 CalculateIncidentSpotLight(SpotLight light, float3 pos, float3 normal, float3 tangent, float3 binormal, MaterialData material) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);
    float3 view_dir = normalize(-pos);

    float cos_between_dir = dot(normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz), light_dir);
    float cos_half_angle = cos(light.cone_angle/2.0);
    float diff = 1.0 - cos_half_angle;
    float factor = saturate((cos_between_dir - cos_half_angle)/diff);

    return BRDF(light_dir, view_dir, normal, tangent, binormal, material) * factor * light.color * light.radiance / distance;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float3 uniform_color = push_constant.color.xyz; 
    float3 ambient_color = uniform_color * lighting_data.ambient_reflection;

    MaterialData material = static_buffer.Load<MaterialData>(push_constant.material_offset);

    float3 color = (float3)0.0; // ambient_color;
    for (uint i = 0; i < lighting_data.num_directional_lights; i++)
    {
        DirectionalLight light = directional_lights[i];
        color += CalculateIncidentDirectionalLight(light, vertex_out.pos, vertex_out.normal, vertex_out.tangent, cross(vertex_out.normal, vertex_out.tangent), material);
    }

    for (i = 0; i < lighting_data.num_omni_directional_lights; i++)
    {
        OmniDirectionalLight light = omni_directional_lights[i];
        color += CalculateIncidentOmniDirectionalLight(light, vertex_out.pos, vertex_out.normal, vertex_out.tangent, cross(vertex_out.normal, vertex_out.tangent), material);
    }

    for (i = 0; i < lighting_data.num_spot_lights; i++)
    {
        SpotLight light = spot_lights[i];
        color += CalculateIncidentSpotLight(light, vertex_out.pos, vertex_out.normal, vertex_out.tangent, cross(vertex_out.normal, vertex_out.tangent), material);
    }

    //color = float3(pow(color.x, 1.0/2.2), pow(color.y, 1.0/2.2), pow(color.z, 1.0/2.2));
    
    float4 result = float4(color, 1.0);
    float4 picking_color = float4(0.0f, 0.5f, 0.5f, 1.0f);

    if (push_constant.is_picked != 0)
        result = result * 0.25f + picking_color * 0.75f;

    return result;
}
