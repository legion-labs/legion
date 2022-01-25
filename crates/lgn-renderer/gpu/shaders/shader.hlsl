#include "crate://lgn-renderer/gpu/pipeline_layout/shader_pipeline_layout.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/entity_transforms.hlsl"
#include "crate://lgn-renderer/gpu/cgen_type/material_data.hlsl"

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

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
float F_Schlick(float f0, float f90, float u) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow(1.0 - u, 5.0);
}

float3 fresnel(float3 f0, float LoH) {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    float f90 = saturate(dot(f0, (float3)(50.0 * 0.33)));

    return f0 + (f90 - f0) * pow(1.0 - LoH, 5.0);
}

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm
//
// simplest approximation
float Fd_Lambert() {
     return 1.0 / 3.14159265358979323846;
}

// vec3 Fd = diffuseColor * Fd_Lambert();
//
// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
float Fd_Burley(float roughness, float NoV, float NoL, float LoH) {
    float f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_Schlick(1.0, f90, NoL);
    float viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / 3.14159265358979323846);
}

// Normal distribution function (specular D)
// Based on https://google.github.io/filament/Filament.html#citation-walter07

// D_GGX(h,α) = α^2 / { π ((n⋅h)^2 (α2−1) + 1)^2 }

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
float D_GGX(float linearRoughness , float NoH) {
    float a = NoH * linearRoughness;
    float k = linearRoughness  / (1.0 - NoH * NoH + a * a);
    float d = k * k * (1.0 / 3.14159265358979323846);
    return d;
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
float V_SmithGGXCorrelated(float linearRoughness, float NoV, float NoL) {
    float a2 = linearRoughness * linearRoughness;
    float GGXV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float GGXL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / (GGXV + GGXL);
}

float3 specular(float roughness, float3 f0, float NoV, float NoL, float NoH, float LoH)
{
	float D = D_GGX(roughness, NoH);
	float V = V_SmithGGXCorrelated(roughness, NoV, NoL);
    float3 F = fresnel(f0, LoH);

	return (D * V) * F;
}

// Remapping [0,1] reflectance to F0
// See https://google.github.io/filament/Filament.html#materialsystem/parameterization/remapping
float3 ComputeF0(float reflectance, float3 base_color, float metallic)
{
 	return (0.08 * reflectance).xxx * (1.0 - metallic) + base_color.rgb * metallic;
}

struct Lighting {
    float3 specular;
    float3 diffuse;
};

Lighting DefaultBRDF(float3 N, float3 V, float3 L, float NoL, MaterialData material, float3 albedo)
{
    float3 H = normalize(L + V);

    float NoV = saturate(dot(N, V));
    float NoH = saturate(dot(N, H));
    float LoH = saturate(dot(L, H));
    float VoH = saturate(dot(V, H));

    float roughness = material.roughness * material.roughness;
    float3 f0 = ComputeF0(material.reflectance, albedo, material.metallic);

    Lighting lighting;
	lighting.diffuse  = NoL * albedo * Fd_Lambert() * (1.0 - material.metallic);
    lighting.specular = NoL * specular(roughness, f0, NoV, NoL, NoH, LoH);
    
    return lighting;
}

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

    float3 albedo = lerp(material.base_color, push_constant.color.xyz, push_constant.color_blend); 

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