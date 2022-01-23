// This is generated file. Do not edit manually

#ifndef TYPE_SPOT_LIGHT
#define TYPE_SPOT_LIGHT

    struct SpotLight {
        float3 pos;
        float radiance;
        float3 dir;
        float cone_angle;
        float3 color;
        uint pad[5];
    };

#endif
