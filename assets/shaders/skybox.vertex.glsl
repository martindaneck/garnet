#version 460 core

layout(location = 0) in vec3 aPos;

out vec3 vDir;

layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2;
};

void main() {
    vDir = aPos;
    // Remove translation from the view matrix
    mat4 view = mat4(mat3(view));
    vec4 clipPos = projection * view * vec4(vDir, 1.0);
    // force depth to 1.0
    gl_Position = clipPos.xyww;
    
}