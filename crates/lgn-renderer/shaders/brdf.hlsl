#include "crate://renderer/codegen/hlsl/cgen_type/material_data.hlsl"

const float PI = 3.14159265358979323846;

float SchlickFresnel(float u)
{
    float m = clamp(1 - u, 0, 1);
    float m2 = m * m;
    return m2 * m2 * m; // pow(m,5)
}

float GTR1(float NdotH, float a)
{
    if (a >= 1) return 1 / PI;

    float a2 = a * a;
    float t = 1 + (a2 - 1) * NdotH * NdotH;
    return (a2 - 1) / (PI * log(a2) * t);
}

float GTR2(float NdotH, float a)
{
    float a2 = a * a;
    float t = 1 + (a2 - 1) * NdotH * NdotH;
    return a2 / (PI * t * t);
}

float GTR2_aniso(float NdotH, float HdotX, float HdotY, float ax, float ay)
{
    float x2 = (HdotX / ax) * (HdotX / ax);
    float y2 = (HdotY / ay) * (HdotY / ay);

    return 1 / (PI * ax*ay * pow(x2 + y2 + NdotH*NdotH, 2));
}

float smithG_GGX(float NdotV, float alphaG)
{
    float a = alphaG * alphaG;
    float b = NdotV * NdotV;
    return 1 / (NdotV + sqrt(a + b - a * b));
}

float smithG_GGX_aniso(float NdotV, float VdotX, float VdotY, float ax, float ay)
{
    float dotX2 = (VdotX * ax) * (VdotX * ax);
    float dotY2 = (VdotY * ay) * (VdotY * ay);

    return 1 / (NdotV + pow(dotX2 + dotY2 + NdotV * NdotV, 2));
}

float3 mon2lin(float3 x)
{
    return float3(pow(x[0], 2.2), pow(x[1], 2.2), pow(x[2], 2.2));
}

float3 BRDF(float3 L, float3 V, float3 N, float3 X, float3 Y, MaterialData material)
{
    float NdotL = dot(N,L);
    float NdotV = dot(N,V);
    if (NdotL < 0 || NdotV < 0) return (float3)0.0;

    float3 H = normalize(L+V);
    float NdotH = dot(N,H);
    float LdotH = dot(L,H);

    float3 Cdlin = mon2lin(material.base_color.xyz);
    float Cdlum = .3 * Cdlin[0] + .6 * Cdlin[1]  + .1 * Cdlin[2]; // luminance approx.

    float3 Ctint = Cdlum > 0 ? Cdlin / Cdlum : (float3)1.0; // normalize lum. to isolate hue+sat
    float3 Cspec0 = lerp(material.specular *.08 * lerp((float3)1.0, Ctint, material.specular_tint), Cdlin, material.metallic);
    float3 Csheen = lerp((float3)1.0, Ctint, material.sheen_tint);

    // Diffuse fresnel - go from 1 at normal incidence to .5 at grazing
    // and lerp in diffuse retro-reflection based on roughness
    float FL = SchlickFresnel(NdotL), FV = SchlickFresnel(NdotV);
    float Fd90 = 0.5 + 2 * LdotH*LdotH * material.roughness;
    float Fd = lerp(1.0, Fd90, FL) * lerp(1.0, Fd90, FV);

    // Based on Hanrahan-Krueger brdf approximation of isotropic bssrdf
    // 1.25 scale is used to (roughly) preserve albedo
    // Fss90 used to "flatten" retroreflection based on roughness
    float Fss90 = LdotH * LdotH * material.roughness;
    float Fss = lerp(1.0, Fss90, FL) * lerp(1.0, Fss90, FV);
    float ss = 1.25 * (Fss * (1.0 / (NdotL + NdotV) - 0.5) + 0.5);

    // specular
    float aspect = sqrt(1-material.anisotropic*.9);
    float ax = max(.001, (material.roughness*material.roughness) / aspect);
    float ay = max(.001, (material.roughness*material.roughness) * aspect);
    float Ds = GTR2_aniso(NdotH, dot(H, X), dot(H, Y), ax, ay);
    float FH = SchlickFresnel(LdotH);
    float3 Fs = lerp(Cspec0, (float3)1.0, FH);
    float Gs;
    Gs  = smithG_GGX_aniso(NdotL, dot(L, X), dot(L, Y), ax, ay);
    Gs *= smithG_GGX_aniso(NdotV, dot(V, X), dot(V, Y), ax, ay);

    // sheen
    float3 Fsheen = FH * material.sheen * Csheen;

    // clearcoat (ior = 1.5 -> F0 = 0.04)
    float Dr = GTR1(NdotH, lerp(.1, .001, material.clearcoat_gloss));
    float Fr = lerp(0.04, 1.0, FH);
    float Gr = smithG_GGX(NdotL, 0.25) * smithG_GGX(NdotV, 0.25); 

    return ((1.0/ PI) * lerp(Fd, ss, material.subsurface)*Cdlin + Fsheen)
        * (1.0 - material.metallic)
        + Gs * Fs * Ds 
        + 0.25 * material.clearcoat * Gr * Fr * Dr;
}
