#version 460 core

struct Light {
        vec4 position_range; // xyz = position, w = range
        vec4 color_intensity; // xyz = color, w = intensity
};

out vec4 FragColor;


in vec2 TexCoords;
in mat3 TBN;
in vec3 FragPos;

layout(std140, binding = 0) uniform Settings {
    float gamma;
    float exposure;
    int environment;
    int rendermode;
    float ssao_radius;
    float ssao_bias;
    int tonemap; // 0: off, 1: on
    float _padding4;
};
layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2; // if problems, change camera_position to a vec4.
};
layout(std140, binding = 2) uniform Lights {
    vec4 count; // count.x is the number of lights, rest is padding
    Light lights[1];
};

layout(binding = 0) uniform sampler2D albedo;
layout(binding = 1) uniform sampler2D normal;
layout(binding = 2) uniform sampler2D orm;

layout(binding = 3) uniform samplerCube irradianceMap;
layout(binding = 4) uniform samplerCube prefilteredMap;
layout(binding = 5) uniform sampler2D brdfLUT;

uniform int screen_width;
uniform int screen_height;
layout(binding = 6) uniform sampler2D ssaoTexture;

// PBR helper functions
vec3 fresnel_schlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = 3.14159265 * denom * denom;

    return num / denom;
}

float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_ggx(NdotV, roughness);
    float ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnel_schlick_roughness(float cosTheta, vec3 F0, float roughness) { // for ambient lighting
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

vec3 rotateY90(vec3 v) {
    return vec3(v.z, v.y, -v.x);
}

vec3 rotateYNegative90(vec3 v) {
    return vec3(-v.z, v.y, v.x);
}

void main() {
    vec4 albedo = texture(albedo, TexCoords);
    vec4 normal_tangent_space = texture(normal, TexCoords);
    vec4 orm = texture(orm, TexCoords);
    vec3 normal = normalize(TBN * (normal_tangent_space.rgb * 2.0 - 1.0));

    vec2 screenUV = gl_FragCoord.xy / vec2(screen_width, screen_height);
    float ssao = texture(ssaoTexture, screenUV).r;

    // PBR 
    vec3 N = normal;
    vec3 V = normalize(camera_position - FragPos);
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo.rgb, orm.b); // orm.b is metallic
    float ao = orm.r; // orm.r is ambient occlusion
    float roughness = orm.g; // orm.g is roughness
    float metallic = orm.b; // orm.b is metallic

    vec3 Lo = vec3(0.0);
    int lightCount = int(count.x);
    for (int i = 0; i < 1; ++i) {
        Light light = lights[i];
        vec3 L = normalize(light.position_range.xyz - FragPos);
        vec3 H = normalize(V + L);
        float distance = length(light.position_range.xyz - FragPos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light.color_intensity.xyz * light.color_intensity.w * attenuation;

        // cook-torrance brdf
        float NDF = distribution_ggx(N, H, roughness);
        float G = geometry_smith(N, V, L, roughness);
        vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001;
        vec3 specular = numerator / denominator;
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;
        kD *= albedo.rgb / 3.14159265;

        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD + specular) * radiance * NdotL;
    }

    /// AMBIENT
    vec3 F = fresnel_schlick_roughness(max(dot(N, V), 0.0), F0, roughness);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    vec3 irradiance = texture(irradianceMap, rotateY90(N)).rgb; // rotate because of some bullshit 
    vec3 diffuse = irradiance * albedo.rgb;
    
    vec3 R = reflect(-V, N);
    const float MAX_REFLECTION_LOD = 6.0; // this value should be tweaked based on the environment map implementation
    vec3 prefilteredColor = textureLod(prefilteredMap, rotateYNegative90(R), roughness * MAX_REFLECTION_LOD).rgb; // rotate because of some bullshit
    vec2 envBRDF = texture(brdfLUT, vec2(max(dot(N, V), 0.0), roughness)).rg;
    vec3 specular = prefilteredColor * (kS * envBRDF.x + envBRDF.y);

    vec3 ambient = (kD * diffuse + specular) * ao;
    ambient *= ssao;

    // final color
    vec3 color = ambient + Lo;

    // different debug modes
    if (rendermode ==  0) { // default
        FragColor = vec4(color, 1.0);
    } else if (rendermode == 1) { // albedo map
        FragColor = vec4(albedo.rgb, 1.0);
    } else if (rendermode == 2) { // normal map
        FragColor = vec4(normal_tangent_space.rgb, 1.0);
    } else if (rendermode == 3) { // ORM map
        FragColor = vec4(orm.rgb, 1.0);
    } else if (rendermode == 4) { // vertex normal
        FragColor = vec4(TBN[2], 1.0);
    } else if (rendermode == 5) { // vertex tangent
        FragColor = vec4(TBN[0], 1.0);
    } else if (rendermode == 6) { // final normal
        FragColor = vec4(normal.rgb, 1.0);
    } else if (rendermode == 7) { // SSAO
        FragColor = vec4(ssao, ssao, ssao, 1.0);
    }
}