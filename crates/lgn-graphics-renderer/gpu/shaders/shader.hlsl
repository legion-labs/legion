#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_transform.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_color.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_picking_data.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/brdf.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 normal : NORMAL;
    float3 tangent : TANGENT;
    float3 pos : POSITION;
    float2 uv_coord : TEXCOORD0;
    nointerpolation uint va_table_address: INSTANCE0;
};

VertexOut main_vs(GpuPipelineVertexIn vertexIn) {
    GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertexIn.va_table_address);
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, addresses.mesh_description_va);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexIn.vertexId);
    VertexOut vertex_out;

    GpuInstanceTransform transform = LoadGpuInstanceTransform(static_buffer, addresses.world_transform_va);
    float3 world_pos = transform_position(transform, vertex_in.pos);
    float3 world_normal = transform_normal(transform, vertex_in.normal);
    float3 world_tangent = transform_normal(transform, vertex_in.tangent);

    float4 pos_view_relative = mul(view_data.view, float4(world_pos, 1.0));

    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    vertex_out.normal = mul(view_data.view, float4(world_normal, 0.0)).xyz;
    vertex_out.tangent = mul(view_data.view, float4(world_tangent, 0.0)).xyz;
    vertex_out.uv_coord = vertex_in.uv_coord;
    vertex_out.va_table_address = vertexIn.va_table_address;
    return vertex_out;
}

Lighting CalculateIncidentDirectionalLight(DirectionalLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = normalize(mul(view_data.view, float4(light.dir, 0.0)).xyz);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
    }

    lighting.diffuse *= light.color * light.radiance;
    lighting.specular *= light.color * light.radiance;

    return lighting;
}

Lighting CalculateIncidentOmniDirectionalLight(OmniDirectionalLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
    }

    lighting.diffuse *= light.color * light.radiance / distance;
    lighting.specular *= light.color * light.radiance / distance;

    return lighting;
}

Lighting CalculateIncidentSpotLight(SpotLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = mul(view_data.view, float4(light.pos, 1.0)).xyz - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
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
    GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertex_out.va_table_address);

    MaterialData material = LoadMaterialData(static_buffer, addresses.material_data_va);
    GpuInstanceColor instance_color = LoadGpuInstanceColor(static_buffer, addresses.instance_color_va);

    LightingMaterial lightingMaterial = (LightingMaterial)0;

    lightingMaterial.albedo = material.base_albedo.rgb;
    if (material.albedo_texture != 0xFFFFFFFF) {
        lightingMaterial.albedo = material_textures[material.albedo_texture].Sample(material_sampler, vertex_out.uv_coord).rgb;
    }
    
    lightingMaterial.metalness = material.base_metalness;
    if (material.metalness_texture != 0xFFFFFFFF) {
        lightingMaterial.metalness = material_textures[material.metalness_texture].Sample(material_sampler, vertex_out.uv_coord).r;
    }

    lightingMaterial.roughness = material.base_roughness;
    if (material.roughness_texture != 0xFFFFFFFF) {
        lightingMaterial.roughness = material_textures[material.roughness_texture].Sample(material_sampler, vertex_out.uv_coord).r;
    }

    lightingMaterial.reflectance = material.reflectance;

    float3 view_normal = vertex_out.normal;
    float3 view_tangent = vertex_out.tangent;
    float3 view_binormal = cross(view_normal, view_tangent);

    float3 lighting_normal = view_normal;
    float3 material_normal = lighting_normal;

    if (material.normal_texture != 0xFFFFFFFF) {
        float3x3 tangent_to_view_space = float3x3(float3(view_tangent.x, view_binormal.x, view_normal.x),
                                                  float3(view_tangent.y, view_binormal.y, view_normal.y),
                                                  float3(view_tangent.z, view_binormal.z, view_normal.z));

        material_normal = material_textures[material.normal_texture].Sample(material_sampler, vertex_out.uv_coord).rgb;

        material_normal = (material_normal * 2.0 - 1);
        material_normal.y *= -1.0;

        lighting_normal = mul(tangent_to_view_space, material_normal);
    }

    lightingMaterial.albedo = lerp(lightingMaterial.albedo.rgb, instance_color.color.rgb, instance_color.color_blend); 

    float3 ambient_color = lightingMaterial.albedo * lighting_data.ambient_reflection;
    float3 diffuse_color = lighting_data.diffuse_reflection.xxx;
    float3 spec_color = lighting_data.specular_reflection.xxx;

    float3 color = ambient_color;
    for (uint i = 0; i < lighting_data.num_directional_lights; i++)
    {
        DirectionalLight light = directional_lights[i];
        Lighting lighting = CalculateIncidentDirectionalLight(light, vertex_out.pos, lighting_normal, lightingMaterial);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    for (uint i = 0; i < lighting_data.num_omni_directional_lights; i++)
    {
        OmniDirectionalLight light = omni_directional_lights[i];
        Lighting lighting = CalculateIncidentOmniDirectionalLight(light, vertex_out.pos, lighting_normal, lightingMaterial);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    for (uint i = 0; i < lighting_data.num_spot_lights; i++)
    {
        SpotLight light = spot_lights[i];
        Lighting lighting = CalculateIncidentSpotLight(light, vertex_out.pos, lighting_normal, lightingMaterial);

        color += diffuse_color * lighting.diffuse;
        color += spec_color * lighting.specular;
    }

    //float3 result = mul(transpose(view_data.view), float4(lighting_normal, 0.0)).rgb;

    return float4(color, 1.0);
}