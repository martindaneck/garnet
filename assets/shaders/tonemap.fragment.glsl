#version 460 core 

in vec2 TexCoords;

out vec4 FragColor;

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

uniform sampler2D hdrTexture;

void main() {
    vec3 hdrColor = texture(hdrTexture, TexCoords).rgb;

    if (tonemap == 0) {
        FragColor = vec4(hdrColor, 1.0);
        return;
    }

    // Apply gamma + exposure tonemapping
    vec3 mapped = vec3(1.0) - exp(-hdrColor * exposure);
    // gamma correction
    mapped = pow(mapped, vec3(1.0 / gamma));

    FragColor = vec4(mapped, 1.0);
}