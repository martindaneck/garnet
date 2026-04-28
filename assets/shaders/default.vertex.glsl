#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec4 aTangent;
layout (location = 3) in vec2 aTexCoords;

out vec2 TexCoords;
out mat3 TBN;
out vec3 FragPos;

layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2;
};

uniform mat4 model;

void main() {
    vec4 positionWorld = model * vec4(aPos, 1.0);
    TexCoords = aTexCoords;
    // tbn matrix
    // aTangent.w is handedness
    vec3 T = normalize(vec3(model * vec4(aTangent.xyz, 0.0)));
    vec3 N = normalize(vec3(model * vec4(aNormal, 0.0)));
    vec3 B = cross(N, T) * aTangent.w;
    TBN = mat3(T, B, N);
    FragPos = positionWorld.xyz;

    gl_Position = projection * view * positionWorld;
}
