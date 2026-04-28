#version 460 core

in vec3 vDir;

out vec4 FragColor;

layout(std140, binding = 0) uniform Settings {
    float gamma;
    float exposure;
    int rendermode;
    float _padding;
};
layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2;
};

uniform samplerCube environmentMap;

void main() {
    //vec3 color = texture(environmentMap, normalize(vDir)).rgb;
    // DEBUG
    vec3 color = textureLod(environmentMap, normalize(vDir), 0.0).rgb;

    FragColor = vec4(color, 1.0);
}
