#include "crate://renderer/codegen/hlsl/cgen_type/material_data.hlsl"

//const float PI = 3.14159265358979323846;

struct BRDFContext
{
	float NoV;
	float NoL;
	float VoL;
	float NoH;
	float VoH;
};

void init(inout BRDFContext context, float3 N, float3 V, float3 L, float NoL)
{
	context.NoL = dot(N, L);
	context.NoV = dot(N, V);
	context.VoL = dot(V, L);
	float invLenH = rsqrt(2 + 2 * context.VoL);
	context.NoH = saturate((context.NoL + context.NoV ) * invLenH);
	context.VoH = saturate(invLenH + invLenH * context.VoL);
}

// Physically based shading model
// parameterized with the below options

// Microfacet specular = D*G*F / (4*NoL*NoV) = D*Vis*F
// Vis = G / (4*NoL*NoV)
float3 diffuse_lambert(float3 albedo)
{
	return albedo * (1 / 3.14159265358979323846);
}

// [Burley 2012, "Physically-Based Shading at Disney"]
float3 diffuse_burley(float3 albedo, float roughness, float NoV, float NoL, float VoH)
{
	float FD90 = 0.5 + 2 * VoH * VoH * roughness;
	float FdV = 1 + (FD90 - 1) * pow(1 - NoV, 5);
	float FdL = 1 + (FD90 - 1) * pow(1 - NoL, 5);
	return albedo * ( (1 / 3.14159265358979323846) * FdV * FdL );
}

// GGX / Trowbridge-Reitz
// [Walter et al. 2007, "Microfacet models for refraction through rough surfaces"]
float D_GGX(float a2, float NoH)
{
	float d = ( NoH * a2 - NoH ) * NoH + 1;	// 2 mad
	return a2 / ( 3.14159265358979323846*d*d );					// 4 mul, 1 rcp
}

// Appoximation of joint Smith term for GGX
// [Heitz 2014, "Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs"]
float vis_smith_joint_approx(float a2, float NoV, float NoL)
{
	float a = sqrt(a2);
	float Vis_SmithV = NoL * ( NoV * ( 1 - a ) + a );
	float Vis_SmithL = NoV * ( NoL * ( 1 - a ) + a );
	return 0.5 * rcp( Vis_SmithV + Vis_SmithL );
}

// [Schlick 1994, "An Inexpensive BRDF Model for Physically-Based Rendering"]
float3 f_schlick(float3 specular_color, float VoH)
{
	float Fc = pow(1 - VoH, 5);					// 1 sub, 3 mul
	//return Fc + (1 - Fc) * SpecularColor;		// 1 add, 3 mad
	
	// Anything less than 2% is physically impossible and is instead considered to be shadowing
	return saturate(50.0 * specular_color.g) * Fc + (1 - Fc) * specular_color;
}

float3 specular_GGX(float roughness, float3 specular_color, BRDFContext context, float NoL)
{
	float a2 = pow(roughness, 4);
	
	// Generalized microfacet specular
	float D = D_GGX(a2, context.NoH);
	float vis = vis_smith_joint_approx(a2, context.NoV, NoL);
	float3 F = f_schlick(specular_color, context.VoH);

	return (D * vis) * F;
}

float dielectric_specular_to_F0(float specular)
{
	return 0.08f * specular;
}

float3 ComputeF0(float specular, float3 base_color, float metallic)
{
	return lerp(dielectric_specular_to_F0(specular).xxx, base_color, metallic.xxx);
}

struct Lighting {
    float3 specular;
    float3 diffuse;
};

Lighting DefaultBRDF(float3 N, float3 V, float3 L, float NoL, MaterialData material, float3 albedo)
{
    BRDFContext context;

	init(context, N, V, L, NoL);

    Lighting lighting;
	lighting.diffuse  = NoL * diffuse_lambert(albedo) * (1.0 -material.metallic);
    lighting.specular = NoL * specular_GGX(material.roughness, ComputeF0(material.specular, albedo, material.metallic), context, NoL);

    return lighting;
}
