#include "crate://lgn-renderer/gpu/cgen_type/material_data.hlsl"

#define PI 3.14159265358979323846

// Fresnel function
float F_Schlick(float f0, float f90, float u) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow(1.0 - u, 5.0);
}

float3 fresnel(float3 f0, float LoH) {
    // f_90 suitable for ambient occlusion
    float f90 = saturate(dot(f0, (float3)(50.0 * 0.33)));
    return f0 + (f90 - f0) * pow(1.0 - LoH, 5.0);
}

// Diffuse BRDF
// simplest approximation
float Fd_Lambert() {
     return 1.0 / PI;
}

// Disney approximation
float Fd_Burley(float roughness, float NoV, float NoL, float LoH) {
    float f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_Schlick(1.0, f90, NoL);
    float viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / PI);
}

// Normal distribution function (specular D)
float D_GGX(float linearRoughness , float NoH) {
    float a = NoH * linearRoughness;
    float k = linearRoughness  / (1.0 - NoH * NoH + a * a);
    float d = k * k * (1.0 / PI);
    return d;
}

// Visibility function (Specular G)
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
float3 ComputeF0(float reflectance, float3 albedo, float metalness)
{
    return 0.16 * reflectance * reflectance * (1.0 - metalness) + albedo * metalness;
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

    float roughness = material.base_roughness * material.base_roughness;
    float3 f0 = ComputeF0(material.reflectance, albedo, material.base_metalness);

    Lighting lighting;
	lighting.diffuse  = NoL * albedo * Fd_Lambert() * (1.0 - material.base_metalness);
    lighting.specular = NoL * specular(roughness, f0, NoV, NoL, NoH, LoH);
    
    return lighting;
}
