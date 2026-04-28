#version 460 core

out vec4 FragColor;

in vec2 TexCoords;
in mat3 TBN;
in vec3 FragPos;

layout(binding = 1) uniform sampler2D normal;

layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2; // if problems, change camera_position to a vec4.
};

void main() {
    vec4 normal_tangent_space = texture(normal, TexCoords);
    vec3 normal = normalize(TBN * (normal_tangent_space.rgb * 2.0 - 1.0));
    normal = normalize(vec3(view * vec4(TBN[2], 0.0))); // to view space
    FragColor = vec4(normal, 1.0);
}


