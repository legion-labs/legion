// #include "crate://renderer/cgen/hlsl/pipeline_layout/default_pipeline_layout.hlsl"

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

struct ConstData {
    float4x4 view;
    float4x4 projection;
    float4 color;
};

struct EntityTransforms {
    float4x4 world;
};

ConstantBuffer<ConstData> const_data;
ByteAddressBuffer static_buffer;

struct PushConstData {
    uint vertex_offset;
    uint world_offset;
    uint is_picked;
};

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    EntityTransforms transform = static_buffer.Load<EntityTransforms>(push_constant.world_offset);
    float4x4 world = transpose(transform.world);

    float4 pos_view_relative = mul(const_data.view, mul(world, vertex_in.pos));
    vertex_out.hpos = mul(const_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    vertex_out.normal = mul(const_data.view, mul(world, vertex_in.normal)).xyz;
    return vertex_out;
}

#define PI 3.141592

struct OmnidirectionalLight {
    float3 pos;
    float radiance;
    float attenuation;
    float3 color;
};

struct DirectionalLight {
    float3 dir;
    float radiance;
    float3 color;
};

struct SpotLight {
    float3 pos;
    float radiance;
    float3 dir;
    float cone_angle;
    float attenuation;
};

struct Lighting {
    float3 specular;
    float3 diffuse;
};

StructuredBuffer<DirectionalLight> directional_lights;
StructuredBuffer<OmnidirectionalLight> omnidirectional_lights;

Lighting CalculateIncidentDirectionalLight(DirectionalLight light, float3 normal, float3 pos) {
    float3 light_dir = normalize(light.dir);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        float3 view_dir = normalize(-pos);
        float3 half_vector = normalize(light_dir + view_dir);
        float spec_angle = max(dot(half_vector, normal), 0.0);
        specular = pow(spec_angle, 16);
    }

    Lighting lighting;
    lighting.diffuse = lambertian * light.color * light.radiance;
    lighting.specular = specular * light.color * light.radiance;

    return lighting;
}

Lighting CalculateIncidentOmnidirectionalLight(OmnidirectionalLight light, float3 normal, float3 pos) {
    float3 light_dir = light.pos - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        float3 view_dir = normalize(-pos);
        float3 half_vector = normalize(light_dir + view_dir);
        float spec_angle = max(dot(half_vector, normal), 0.0);
        specular = pow(spec_angle, 16);
    }

    Lighting lighting;
    lighting.diffuse = lambertian * light.color * light.radiance / distance;
    lighting.specular = specular * light.color * light.radiance / distance;

    return lighting;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 uniform_color = const_data.color; 
    float3 ambient_color = uniform_color * 0.01;
    float3 diffuse_color = uniform_color.xyz;
    float3 spec_color = diffuse_color + float3(0.5, 0.5, 0.5);

    float3 color = ambient_color;
    uint num_lights, stride_lights;
    directional_lights.GetDimensions(num_lights, stride_lights);
    for (uint i = 0; i < num_lights; i++)
    {
        DirectionalLight light = directional_lights[i];
        Lighting lighting = CalculateIncidentDirectionalLight(light, vertex_out.normal, vertex_out.pos);
        color += diffuse_color * lighting.diffuse + spec_color * lighting.specular;
    }

    omnidirectional_lights.GetDimensions(num_lights, stride_lights);
    for (i = 0; i < num_lights; i++)
    {
        OmnidirectionalLight light = omnidirectional_lights[i];
        Lighting lighting = CalculateIncidentOmnidirectionalLight(light, vertex_out.normal, vertex_out.pos);
        color += diffuse_color * lighting.diffuse + spec_color * lighting.specular;
    }
    
    float4 result = float4(pow(color, float3(1.0/2.2, 1.0/2.2, 1.0/2.2)), 1.0);
    float4 picking_color = float4(0.0f, 0.5f, 0.5f, 1.0f);

    if (push_constant.is_picked != 0)
        result = result * 0.25f + picking_color * 0.75f;

    return result;
}