// #include "crate://renderer/cgen/hlsl/pipeline_layout/default_pipeline_layout.hlsl"

struct VertexIn {
    float3 pos : POSITION;
    float3 normal : NORMAL;   
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 normal : NORMAL;
    float3 pos : POSITION;
};

 struct ConstData {
     float4x4 world;
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
    uint offset;
};

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;

    EntityTransforms transform = static_buffer.Load<EntityTransforms>(push_constant.offset);
    float4x4 world = transpose(transform.world);

    float4 pos_view_relative = mul(const_data.view, mul(world, float4(vertex_in.pos, 1.0)));
    vertex_out.hpos = mul(const_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    vertex_out.normal = mul(const_data.view, mul(world, float4(vertex_in.normal, 0.0))).xyz;
    return vertex_out;
}

#define PI 3.141592

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float3 normal = normalize(vertex_out.normal);
    float3 light_pos = float3(1.0, 4.0, -2.0);
    float3 light_dir = light_pos - vertex_out.pos.xyz;
    float3 light_color = float3(1.0, 1.0, 1.0);
    float light_power = 40.0;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    float4 uniform_color = const_data.color; 

    float3 ambient_color = uniform_color.xyz / 5.0;
    float3 diffuse_color = uniform_color.xyz;
    float3 spec_color = float3(1.0, 1.0, 1.0);

    float lambertian = max(dot(light_dir, normal)/PI, 0.0);
    float specular = 0.0;

    if (lambertian > 0.0)
    {
        float3 view_dir = normalize(-vertex_out.pos.xyz);
        float3 half_vector = normalize(light_dir + view_dir);
        float spec_angle = max(dot(half_vector, vertex_out.normal), 0.0);
        float specular = pow(spec_angle, 16);
    }

    float3 color = ambient_color + 
                    diffuse_color * lambertian * light_color * light_power / distance + 
                    spec_color * specular * light_color * light_power / distance;
    //debug normals: float4((float3(1.0, 1.0, 1.0) + vertex_out.normal)/2.0, 1.0);
    return float4(pow(color, float3(1.0/2.2, 1.0/2.2, 1.0/2.2)), 1.0);
}