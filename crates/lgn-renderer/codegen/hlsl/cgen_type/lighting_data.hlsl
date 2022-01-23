// This is generated file. Do not edit manually

#ifndef TYPE_LIGHTING_DATA
#define TYPE_LIGHTING_DATA

    struct LightingData {
        uint num_directional_lights;
        uint num_omni_directional_lights;
        uint num_spot_lights;
        uint diffuse;
        uint specular;
        float specular_reflection;
        float diffuse_reflection;
        float ambient_reflection;
        float shininess;
        uint3 pad_;
    };

#endif
