#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_color.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_picking_data.hlsl"
#include "crate://lgn-graphics-renderer/gpu/cgen_type/gpu_instance_va_table.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/brdf.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/transform.hsh"

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 normal : NORMAL;
    float4 tangent : TANGENT;
    float3 pos : POSITION;
    float2 uv_coord : TEXCOORD0;
    nointerpolation uint va_table_address: INSTANCE0;
};

VertexOut main_vs(GpuPipelineVertexIn vertexIn) {
    GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertexIn.va_table_address);
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, addresses.mesh_description_va);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, addresses.mesh_description_va, vertexIn.vertexId);
    VertexOut vertex_out = (VertexOut)0;

    TransformData transform = LoadTransformData(static_buffer, addresses.world_transform_va);
    float3 world_pos = ((Transform)transform).apply_to_point(vertex_in.pos);
    float3 view_pos = transform_from_tr(view_data.camera_translation, view_data.camera_rotation).apply_to_point(world_pos);

    vertex_out.hpos = mul(view_data.projection, float4(view_pos, 1.0));
    vertex_out.pos = view_pos;
    float3 normal_in_world = normalize(((Transform)transform).apply_to_vector(vertex_in.normal));
    vertex_out.normal = normalize(transform_from_rotation(view_data.camera_rotation).apply_to_vector(normal_in_world));
    float3 tangent_in_world = normalize(((Transform)transform).apply_to_vector(vertex_in.tangent.xyz));
    vertex_out.tangent = float4(normalize(transform_from_rotation(view_data.camera_rotation).apply_to_vector(tangent_in_world)), vertex_in.tangent.w);
    vertex_out.uv_coord = vertex_in.uv_coord;
    vertex_out.va_table_address = vertexIn.va_table_address;
    return vertex_out;
}

Lighting CalculateIncidentDirectionalLight(DirectionalLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = normalize(transform_from_rotation(view_data.camera_rotation).apply_to_vector(light.dir));

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
    }

    lighting.diffuse *= unpack_srgb2linear( light.color).rgb * light.radiance;
    lighting.specular *= unpack_srgb2linear( light.color).rgb * light.radiance;

    return lighting;
}

Lighting CalculateIncidentOmniDirectionalLight(OmniDirectionalLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = transform_from_tr(view_data.camera_translation, view_data.camera_rotation).apply_to_point(light.pos) - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
    }

    lighting.diffuse *= unpack_srgb2linear( light.color).rgb * light.radiance / distance;
    lighting.specular *= unpack_srgb2linear( light.color).rgb * light.radiance / distance;

    return lighting;
}

Lighting CalculateIncidentSpotLight(SpotLight light, float3 pos, float3 normal, LightingMaterial material) {
    float3 light_dir = transform_from_tr(view_data.camera_translation, view_data.camera_rotation).apply_to_point(light.pos) - pos;
    float distance = length(light_dir);
    distance = distance * distance;
    light_dir = normalize(light_dir);

    Lighting lighting = (Lighting)0;
    float NoL = saturate(dot(normal, light_dir));
    if (NoL > 0)
    {
        lighting = DefaultBRDF(normal, normalize(-pos), light_dir, NoL, material);
    }

    float cos_between_dir = dot(normalize(transform_from_rotation(view_data.camera_rotation).apply_to_vector(light.dir)), light_dir);
    float cos_half_angle = cos(light.cone_angle/2.0);
    float diff = 1.0 - cos_half_angle;
    float factor = saturate((cos_between_dir - cos_half_angle)/diff);

    lighting.diffuse *= factor * unpack_srgb2linear( light.color).rgb * light.radiance / distance;
    lighting.specular *= factor * unpack_srgb2linear( light.color).rgb * light.radiance / distance;

    return lighting;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    GpuInstanceVATable addresses = LoadGpuInstanceVATable(static_buffer, vertex_out.va_table_address);

    MaterialData material = LoadMaterialData(static_buffer, addresses.material_data_va);
    GpuInstanceColor instance_color = LoadGpuInstanceColor(static_buffer, addresses.instance_color_va);

    LightingMaterial lightingMaterial = (LightingMaterial)0;

    lightingMaterial.albedo = material.base_albedo.rgb;
    if (material.albedo_texture != 0xFFFFFFFF) {
        lightingMaterial.albedo = material_textures[material.albedo_texture].Sample(material_samplers[material.sampler], vertex_out.uv_coord).rgb;
    }
    
    lightingMaterial.metalness = material.base_metalness;
    if (material.metalness_texture != 0xFFFFFFFF) {
        lightingMaterial.metalness = material_textures[material.metalness_texture].Sample(material_samplers[material.sampler], vertex_out.uv_coord).r;
    }

    lightingMaterial.roughness = material.base_roughness;
    if (material.roughness_texture != 0xFFFFFFFF) {
        lightingMaterial.roughness = material_textures[material.roughness_texture].Sample(material_samplers[material.sampler], vertex_out.uv_coord).r;
    }

    lightingMaterial.reflectance = material.reflectance;

    float3 view_normal = vertex_out.normal;
    float3 view_tangent = vertex_out.tangent.xyz;
    float sign = vertex_out.tangent.w;
    float3 view_binormal = sign * cross(view_normal, view_tangent);

    float3 lighting_normal = view_normal;
    float3 material_normal = lighting_normal;

    if (material.normal_texture != 0xFFFFFFFF) {
        float3x3 tangent_to_view_space = float3x3(float3(view_tangent.x, view_binormal.x, view_normal.x),
                                                  float3(view_tangent.y, view_binormal.y, view_normal.y),
                                                  float3(view_tangent.z, view_binormal.z, view_normal.z));

        material_normal = material_textures[material.normal_texture].Sample(material_samplers[material.sampler], vertex_out.uv_coord).rgb;

        material_normal = (material_normal * 2.0 - 1);

        lighting_normal = mul(tangent_to_view_space, material_normal);
    }

    lightingMaterial.albedo = lerp(lightingMaterial.albedo.rgb, unpack_srgb2linear( instance_color.color).rgb, instance_color.color_blend); 

    const float3 ambient_color = lighting_data.default_ambient;
    const float3 diffuse_color = lighting_data.apply_diffuse ?  float3(1.f,1.f,1.f) : float3(0.f,0.f,0.f);
    const float3 spec_color = lighting_data.apply_specular ?  float3(1.f,1.f,1.f) : float3(0.f,0.f,0.f);

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

    return float4(color, 1.0);
}